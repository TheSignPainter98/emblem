#[macro_use]
mod log;

mod args;
mod ast;
mod build;
mod context;
mod init;
mod parser;
mod repo;

use args::{Args, Command};
use log::Log;
use std::process::ExitCode;

fn main() -> ExitCode {
    let args = Args::parse();
    log::init(args.log);

    let ret = match args.command {
        Command::Build(args) => build::build(args),
        Command::Format(_) => panic!("fmt not implemented"),
        Command::Init(args) => init::init(args),
        Command::Lint(_) => panic!("lint not implemented"),
        Command::List(_) => panic!("list not implemented"),
    };
    if let Err(e) = ret {
        alert!(Log::error(&e.to_string()));
    }

    log::report()
}
