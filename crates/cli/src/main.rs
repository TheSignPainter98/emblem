#[cfg(test)]
#[macro_use]
extern crate pretty_assertions;

mod init;
mod manifest;

pub use crate::init::Initialiser;
use arg_parser::{Args, Command};
use emblem_core::{
    context::{Dependency, DependencyName},
    log::Logger,
    Action, Builder, Context, Explainer, Linter, Log,
};
use itertools::Itertools;
use std::{collections::HashMap, fs, process::ExitCode};

fn main() -> ExitCode {
    let args = Args::parse();

    let mut ctx = Context::new();

    let mut logger = Logger::new(
        args.log.verbosity.into(),
        args.log.colour,
        args.log.warnings_as_errors,
    );

    let raw_manifest = match fs::read_to_string("emblem.yml") {
        Ok(m) => m,
        Err(e) => {
            Log::error(e.to_string()).print(&mut logger);
            return ExitCode::FAILURE;
        }
    };
    if let Err(e) = load_manifest(&mut ctx, &raw_manifest, &args) {
        e.print(&mut logger);
        return ExitCode::FAILURE;
    };

    let warnings_as_errors = args.log.warnings_as_errors;
    let (logs, successful) = match &args.command {
        Command::Add(args) => todo!("{:?}", args),
        Command::Build(args) => execute(&mut ctx, Builder::from(args), warnings_as_errors),
        Command::Explain(args) => execute(&mut ctx, Explainer::from(args), warnings_as_errors),
        Command::Format(_) => todo!(),
        Command::Init(args) => execute(&mut ctx, Initialiser::from(args), warnings_as_errors),
        Command::Lint(args) => execute(&mut ctx, Linter::from(args), warnings_as_errors),
        Command::List(_) => todo!(),
    };
    for log in logs {
        log.print(&mut logger);
    }

    logger.report();

    if successful {
        ExitCode::SUCCESS
    } else {
        ExitCode::FAILURE
    }
}

fn load_manifest<'ctx, 'm, 'a>(
    ctx: &'ctx mut Context<'m>,
    src: &'m str,
    args: &'a Args,
) -> Result<(), Box<Log<'m>>>
where
    'm: 'ctx,
    'a: 'm,
{
    let manifest = manifest::load_str(src)?;

    let doc_info = ctx.doc_info_mut();
    doc_info.set_name(manifest.name);
    doc_info.set_emblem_version(manifest.emblem_version.into());

    if let Some(authors) = manifest.authors {
        doc_info.set_authors(authors);
    }

    if let Some(keywords) = manifest.keywords {
        doc_info.set_keywords(keywords);
    }

    let lua_info = ctx.lua_info_mut();

    let mut specific_args: HashMap<_, Vec<_>> = HashMap::new();
    if let Some(lua_args) = args.extension_args() {
        lua_info.set_sandbox(lua_args.sandbox.into());
        lua_info.set_max_mem(lua_args.max_mem.into());

        let mut general_args = Vec::with_capacity(lua_args.args.len());
        for arg in &lua_args.args {
            let name = arg.name();

            match name.find('.') {
                None => general_args.push((name, arg.value())),
                Some(0) => {
                    return Err(Box::new(Log::error(format!(
                        "argument module name cannot be empty: got '{}' in '{}={}'",
                        name, name, arg.value(),
                    ))))
                }
                Some(idx) => {
                    let dep_name = &name[..idx];
                    let arg_name = &name[1+idx..];
                    if let Some(args) = specific_args.get_mut(dep_name) {
                        args.push((arg_name, arg.value()));
                    } else {
                        specific_args.insert(dep_name, vec![(arg_name, arg.value())]);
                    }
                }
            }
        }

        lua_info.set_general_args(general_args);
    }

    let dependencies = manifest
        .requires
        .unwrap_or_default()
        .into_iter()
        .map(|(k, v)| {
            let k: DependencyName<'m> = k.into();
            let mut v: Dependency<'m> = v.into();
            if let Some(args) = specific_args.remove(v.rename_as().unwrap_or(k.name())) {
                let dep_args = v.args_mut();
                for (k2, v2) in args {
                    dep_args.insert(k2, v2);
                }
            }
            (k, v)
        })
        .collect();

    if !specific_args.is_empty() {
        return Err(Box::new(Log::error(format!(
            "Unused arguments: {}",
            specific_args.keys().join(", ")
        ))));
    }

    ctx.set_dependencies(dependencies);

    Ok(())
}

fn execute<'m, C, R>(
    ctx: &'m mut Context<'m>,
    cmd: C,
    warnings_as_errors: bool,
) -> (Vec<Log<'_>>, bool)
where
    C: Action<Response = R>,
{
    let mut run_res = cmd.run(ctx);

    if !run_res.successful(warnings_as_errors) {
        (run_res.logs, false)
    } else {
        let output_res = cmd.output(run_res.response);
        let successful = output_res.successful(warnings_as_errors);

        run_res.logs.extend(output_res.logs);
        (run_res.logs, successful)
    }
}
