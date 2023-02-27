use arg_parser::{Args, Command};
use emblem_core::{log::Logger, Action, Builder, Context, Explainer, Initialiser, Linter};
use std::process::ExitCode;

fn main() -> ExitCode {
    let args = Args::parse();

    let cmd: Box<dyn Action> = match args.command {
        Command::Build(args) => Box::new(Builder::from(args)),
        Command::Explain(args) => Box::new(Explainer::from(args)),
        Command::Format(_) => todo!(),
        Command::Init(args) => Box::new(Initialiser::from(args)),
        Command::Lint(args) => Box::new(Linter::from(args)),
        Command::List(_) => todo!(),
    };

    let mut ctx = Context::new();
    let msgs = cmd.run(&mut ctx);

    let mut logger = Logger::new(
        args.log.verbosity.into(),
        args.log.colour,
        args.log.warnings_as_errors,
    );
    for msg in msgs {
        msg.print(&mut logger);
    }
    logger.report();
    logger.exit_code()
}
