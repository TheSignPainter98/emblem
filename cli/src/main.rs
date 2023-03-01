use arg_parser::{Args, Command};
use emblem_core::{log::Logger, Action, Builder, Context, Explainer, Initialiser, Linter, Log};
use std::process::ExitCode;

fn main() -> ExitCode {
    let args = Args::parse();
    let warnings_as_errors = args.log.warnings_as_errors;

    let cmd: Box<dyn Action> = match args.command {
        Command::Build(args) => Box::new(Builder::from(args)),
        Command::Explain(args) => Box::new(Explainer::from(args)),
        Command::Format(_) => todo!(),
        Command::Init(args) => Box::new(Initialiser::from(args)),
        Command::Lint(args) => Box::new(Linter::from(args)),
        Command::List(_) => todo!(),
    };

    let mut ctx = Context::new();
    let result = cmd.run(&mut ctx);
    let successful = result.successful(warnings_as_errors);

    let mut logger = Logger::new(
        args.log.verbosity.into(),
        args.log.colour,
        warnings_as_errors,
    );
    for log in result.logs {
        log.print(&mut logger);
    }

    if let Err(e) = result.result.output() {
        Log::error(e.to_string()).print(&mut logger);
    }

    logger.report();

    if successful {
        ExitCode::SUCCESS
    } else {
        ExitCode::FAILURE
    }
}
