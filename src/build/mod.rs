use crate::args::{BuildCmd, SearchResult};
use crate::context::Context;
use crate::log::{Log, Src};
use crate::parser::{self, Error};
use std::error;

pub fn build(cmd: BuildCmd) -> Result<(), Box<dyn error::Error>> {
    let mut ctx = Context::new();

    let fname = SearchResult::try_from(&cmd.input.file)?;

    match parser::parse_file(&mut ctx, fname) {
        Ok(d) => println!("{:?}", d),
        Err(e) => {
            match *e {
                Error::StringConversion(e) => alert!(Log::error(&e.to_string())),
                Error::Filesystem(e) => alert!(Log::error(&e.to_string())),
                Error::Parse(e) =>  alert!(Log::error(&e.to_string()))
            }
        }
    }

    Ok(())
}
