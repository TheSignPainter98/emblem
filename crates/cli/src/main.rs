#[cfg(test)]
#[macro_use]
extern crate pretty_assertions;

mod init;
mod manifest;

pub use crate::init::Initialiser;
use arg_parser::{Args, Command};
use emblem_core::{log::Logger, Action, Builder, Context, Explainer, Linter, Log, context::{Dependency, DependencyName}};
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
    if let Err(e) = load_manifest(&mut ctx, &raw_manifest) {
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

fn load_manifest<'ctx, 'm>(ctx: &'ctx mut Context<'m>, src: &'m str) -> Result<(), Box<Log<'m>>>
where
    'm: 'ctx,
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

    if let Some(dependencies) = manifest.requires {
        ctx.set_dependencies(
            dependencies
                .into_iter()
                .map(|(k, v)| (k, v.into()))
                .collect(),
        );
    }

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
