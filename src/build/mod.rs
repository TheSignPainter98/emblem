use crate::args::{BuildCmd, SearchResult};
use crate::context::Context;
use crate::parser;
use std::error;

pub fn build(cmd: BuildCmd) -> Result<(), Box<dyn error::Error>> {
    let mut ctx = Context::new();

    let fname: SearchResult = cmd.input.file.as_ref().try_into()?;

    match parser::parse_file(&mut ctx, fname) {
        Ok(d) => println!("{:?}", d),
        Err(errs) => for e in errs {
            alert!(e);
        },
    }

    Ok(())
}
