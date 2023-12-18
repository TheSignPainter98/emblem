#[cfg(test)]
#[macro_use]
extern crate pretty_assertions;

mod error;
mod init;
mod manifest;
mod pretty_logger;
mod result;

pub use crate::error::Error;
pub use crate::result::Result;

use crate::init::Initialiser;
use crate::pretty_logger::PrettyLogger;
use arg_parser::{Args, Command};
use emblem_core::{log::Logger, Action, Builder, Context, Explainer, Linter};
use manifest::DocManifest;
use std::{collections::HashMap, fs, process::ExitCode};

fn main() -> ExitCode {
    let args = Args::parse();

    let logger = PrettyLogger::builder()
        .verbosity(args.log.verbosity)
        .max_errors(args.log.max_errors)
        .colourise(args.log.colour)
        .build()
        .expect("internal error: failed to build pretty logger");
    let mut ctx = Context::new(logger).warnings_as_errors(args.log.warnings_as_errors);

    let r = execute(&mut ctx, &args);
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

fn execute<L: Logger>(ctx: &mut Context<L>, args: &Args) -> Result<()> {
    match &args.command {
        Command::Add(add_args) => todo!("{:?}", add_args), // integrate_manifest!() here
        Command::Build(build_args) => {
            load_manifest(ctx, "emblem.toml", args)?; // TODO(kcza): search parents for the
                                                      // manifest; find lock file in same location
            Ok(Builder::from(build_args).run(ctx).map(|_| ())?)
        }
        Command::Explain(explain_args) => Ok(Explainer::from(explain_args).run(ctx)?),
        Command::Format(_) => todo!(),
        Command::Init(init_args) => Initialiser::from(init_args).run(ctx),
        Command::Lint(lint_args) => Ok(Linter::from(lint_args).run(ctx)?),
        Command::List(_) => todo!(), // integrate_manifest!()  here
    }
}

fn load_manifest<L: Logger>(ctx: &mut Context<L>, src: &str, args: &Args) -> Result<()> {
    // TODO(kcza): improve error log here!
    let manifest = DocManifest::try_from(
        fs::read_to_string(src)
            .map_err(|e| Error::io(src, e))?
            .as_ref(),
    )?;
    ctx.set_name(manifest.metadata.name);
    ctx.set_version(manifest.metadata.version.into());

    let doc_info = ctx.doc_params_mut();
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
