#[macro_use]
mod log;

#[cfg(test)]
#[macro_use]
extern crate pretty_assertions;

mod args;
mod ast;
mod build;
mod context;
mod explain;
mod init;
mod lint;
mod parser;
mod repo;
mod util;

use args::{Args, Command};
use log::Log;
use std::process::ExitCode;

fn main() -> ExitCode {
    let args = Args::parse();
    log::init(args.log);

    let ret = match args.command {
        Command::Build(args) => build::build(args),
        Command::Explain(args) => explain::explain(args),
        Command::Format(_) => panic!("fmt not implemented"),
        Command::Init(args) => init::init(args),
        Command::Lint(args) => lint::lint(args),
        Command::List(_) => panic!("list not implemented"),
    };
    if let Err(e) = ret {
        alert!(Log::error(&e.to_string()));
    }

    log::report()
}
