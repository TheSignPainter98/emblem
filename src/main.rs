mod args;
mod ast;
mod build;
mod context;
mod init;
mod parser;

use args::{Args, Command};
use std::error::Error;

fn main() {
    let args = Args::parse();
    exec(args).unwrap_or_else(|e| panic!("error: {}", e));
}

fn exec(args: Args) -> Result<(), Box<dyn Error>> {
    match args.command {
        Command::Build(args) => build::build(args),
        Command::Format(_) => panic!("fmt not implemented"),
        Command::Init(args) => init::init(args),
        Command::Lint(_) => panic!("lint not implemented"),
        Command::List(_) => panic!("list not implemented"),
    }
}
