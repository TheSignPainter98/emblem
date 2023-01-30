mod args;
mod ast;
mod build;
mod context;
mod init;
mod log;
mod parser;
mod repo;

use args::{Args, Command};
use std::process::ExitCode;

fn main() -> ExitCode {
    let args = Args::parse();
    log::init(args.log);

    match args.command {
        Command::Build(args) => build::build(args),
        Command::Format(_) => panic!("fmt not implemented"),
        Command::Init(args) => init::init(args),
        Command::Lint(_) => panic!("lint not implemented"),
        Command::List(_) => panic!("list not implemented"),
    }.unwrap();

    log::report()
}
