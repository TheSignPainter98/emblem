use crate::args::BuildCmd;
use crate::context::Context;
use crate::parser;
use std::error::Error;

pub fn build(cmd: BuildCmd) -> Result<(), Box<dyn Error>> {
    let mut ctx = Context::new();

    if let Some(fname) = cmd.input.file.path() {
        // parser::parse_file(&mut ctx, fname.to_str().unwrap().to_owned())?;
        Ok(())
    } else {
        Err(Box::new(std::io::Error::new(
            std::io::ErrorKind::InvalidInput,
            "please specify an input",
        )))
    }
}
