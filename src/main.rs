mod args;
mod init;

use args::{Args, Command};
use std::error::Error;

fn main() {
    let args = Args::parse();
    exec(args).unwrap_or_else(|e| panic!("error: {}", e));
}

fn exec(args: Args) -> Result<(), Box<dyn Error>> {
    match args.command {
        Command::Build(_) => panic!("build not implemented"),
        Command::Format(_) => panic!("fmt not implemented"),
        Command::Init(args) => init::init(args),
        Command::Lint(_) => panic!("lint not implemented"),
        Command::List(_) => panic!("list not implemented"),
    }
}
