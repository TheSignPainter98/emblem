use crate::args::{BuildCmd, SearchResult};
use crate::context::Context;
use crate::parser;
use std::error::Error;

pub fn build(cmd: BuildCmd) -> Result<(), Box<dyn Error>> {
    let mut ctx = Context::new();

    let fname = SearchResult::try_from(&cmd.input.file)?;
    match parser::parse_file(&mut ctx, fname) {
        Ok(d) => println!("{:?}", d),
        Err(e) => println!("{}", e),
    }

    Ok(())
}
