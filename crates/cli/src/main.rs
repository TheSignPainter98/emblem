#[cfg(test)]
#[macro_use]
extern crate pretty_assertions;

mod error;
mod init;
mod manifest;
mod result;

pub use crate::error::Error;
pub use crate::result::Result;

use crate::init::Initialiser;
use arg_parser::{Args, Command};
use emblem_core::{log::Logger, Action, Builder, Context, Explainer, Linter};
use manifest::DocManifest;
use std::{collections::HashMap, fs, process::ExitCode};

fn main() -> ExitCode {
    let args = Args::parse();
    let logger = Logger::new(
        args.log.verbosity.into(),
        args.log.colour,
        args.log.warnings_as_errors,
    );
    let mut ctx = Context::new_with_logger(logger);
    let mut raw_manifest = String::new();
    let r = execute(&mut raw_manifest, &mut ctx, &args);
    let success = r.is_ok();
    if let Err(e) = r {
        ctx.print(e).ok();
    }

    ctx.report()
        .expect("internal error: failed to output report");

    if success {
        ExitCode::SUCCESS
    } else {
        ExitCode::FAILURE
    }
}

fn execute<'a, 'b>(raw_manifest: &'a mut String, ctx: &'b mut Context, args: &Args) -> Result<()>
where
    'a: 'b,
{
    match &args.command {
        Command::Add(add_args) => todo!("{:?}", add_args), // integrate_manifest!() here
        Command::Build(build_args) => {
            *raw_manifest = fs::read_to_string("emblem.yml")?;
            load_manifest(ctx, raw_manifest, args)?;
            Ok(Builder::from(build_args).run(ctx).map(|_| ())?)
        }
        Command::Explain(explain_args) => Ok(Explainer::from(explain_args).run(ctx)?),
        Command::Format(_) => todo!(),
        Command::Init(init_args) => Initialiser::from(init_args).run(ctx),
        Command::Lint(lint_args) => Ok(Linter::from(lint_args).run(ctx)?),
        Command::List(_) => todo!(), // integrate_manifest!()  here
    }
}

fn load_manifest(ctx: &mut Context, src: &str, args: &Args) -> Result<()> {
    let manifest = DocManifest::try_from(src)?;

    let doc_info = ctx.doc_params_mut();
    doc_info.set_name(manifest.metadata.name);
    doc_info.set_emblem_version(manifest.metadata.emblem_version.into());

    if let Some(authors) = manifest.metadata.authors {
        doc_info.set_authors(authors);
    }

    if let Some(keywords) = manifest.metadata.keywords {
        doc_info.set_keywords(keywords);
    }

    let lua_info = ctx.lua_params_mut();

    let mut specific_args: HashMap<_, Vec<_>> = HashMap::new();
    if let Some(lua_args) = args.lua_args() {
        lua_info.set_sandbox_level(lua_args.sandbox_level.into());
        lua_info.set_max_mem(lua_args.max_mem.into());
        lua_info.set_max_steps(lua_args.max_steps.into());

        let mut general_args = Vec::with_capacity(lua_args.args.len());
        for arg in &lua_args.args {
            let name = arg.name();

            match name.find('.') {
                None => general_args.push((name.to_string(), arg.value().to_string())),
                Some(0) => {
                    return Err(Error::arg_invalid(
                        arg.value().to_string(),
                        "module name cannot be empty",
                    ));
                }
                Some(idx) => {
                    let dep_name = &name[..idx];
                    let arg_name = &name[1 + idx..];
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

    let modules = manifest
        .dependencies
        .unwrap_or_default()
        .into_iter()
        .map(|(name, module)| {
            let mut module = module.into_module(name.clone());
            if let Some(args) = specific_args.remove(module.rename_as().unwrap_or(&name)) {
                let dep_args = module.args_mut();
                for (k2, v2) in args {
                    dep_args.insert(k2.to_string(), v2.to_string());
                }
            }
            module
        })
        .collect();

    if !specific_args.is_empty() {
        return Err(Error::unused_args(
            specific_args.keys().map(ToString::to_string).collect(),
        ));
    }

    lua_info.set_modules(modules);

    Ok(())
}
