use crate::args::{BuildCmd, SearchResult};
use crate::context::Context;
use crate::log::messages::{UnexpectedEOF, UnexpectedToken};
use crate::log::Log;
use crate::parser::{self, error::LalrpopError, Error, Location};
use std::error;

pub fn build(cmd: BuildCmd) -> Result<(), Box<dyn error::Error>> {
    let mut ctx = Context::new();

    let fname: SearchResult = cmd.input.file.as_ref().try_into()?;

    match parser::parse_file(&mut ctx, fname) {
        Ok(d) => println!("{:?}", d),
        Err(e) => match *e {
            Error::StringConversion(e) => alert!(Log::error(&e.to_string())),
            Error::Filesystem(e) => alert!(Log::error(&e.to_string())),
            Error::Parse(e) => match e {
                LalrpopError::InvalidToken { location } => {
                    panic!("internal error: invalid token at {}", location)
                }
                LalrpopError::UnrecognizedEOF { location, expected } => {
                    alert!(UnexpectedEOF::new(location, expected))
                }
                LalrpopError::UnrecognizedToken {
                    token: (l, t, r),
                    expected,
                } => {
                    alert!(UnexpectedToken::new(Location::new(&l, &r), t, expected));
                }
                LalrpopError::ExtraToken { token: (l, t, r) } => panic!(
                    "internal error: extra token {} at {}",
                    t,
                    Location::new(&l, &r)
                ),
                LalrpopError::User { error } => {
                    alert!(error);
                }
            },
        },
    }

    Ok(())
}
