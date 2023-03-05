use arg_parser::{Args, Command};
use emblem_core::{log::Logger, Action, Builder, Context, Explainer, Initialiser, Linter, Log};
use std::process::ExitCode;

fn main() -> ExitCode {
    let args = Args::parse();
    let warnings_as_errors = args.log.warnings_as_errors;

    let mut ctx = Context::new();

    let (logs, successful) = match args.command {
        Command::Build(args) => execute(&mut ctx, Builder::from(args), warnings_as_errors),
        Command::Explain(args) => execute(&mut ctx, Explainer::from(args), warnings_as_errors),
        Command::Format(_) => todo!(),
        Command::Init(args) => execute(&mut ctx, Initialiser::from(args), warnings_as_errors),
        Command::Lint(args) => execute(&mut ctx, Linter::from(args), warnings_as_errors),
        Command::List(_) => todo!(),
    };

    let mut logger = Logger::new(
        args.log.verbosity.into(),
        args.log.colour,
        warnings_as_errors,
    );
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

fn execute<C, R>(ctx: &mut Context, cmd: C, warnings_as_errors: bool) -> (Vec<Log<'_>>, bool)
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
