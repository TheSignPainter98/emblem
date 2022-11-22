use crate::args::BuildCmd;
use crate::parser;
use std::error::Error;

pub fn build(cmd: BuildCmd) -> Result<(), Box<dyn Error>> {
    if let Some(fname) = cmd.input.file.path() {
        parser::parse(fname.as_ref())?;
        Ok(())
    } else {
        Err(Box::new(std::io::Error::new(
            std::io::ErrorKind::InvalidInput,
            "please specify an input",
        )))
    }
}
