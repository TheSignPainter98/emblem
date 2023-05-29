use crate::RawArgs;
use clap::{
    builder::{StringValueParser, TypedValueParser},
    error::{Error as ClapError, ErrorKind as ClapErrorKind},
    CommandFactory,
};

/// Command-line arg declaration
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ExtArg {
    raw: String,
    eq_idx: usize,
}

impl ExtArg {
    pub(crate) fn parser() -> impl TypedValueParser {
        StringValueParser::new().try_map(Self::try_from)
    }

    #[allow(dead_code)]
    pub fn name(&self) -> &str {
        &self.raw[..self.eq_idx]
    }

    #[allow(dead_code)]
    pub fn value(&self) -> &str {
        &self.raw[self.eq_idx + 1..]
    }
}

impl TryFrom<String> for ExtArg {
    type Error = ClapError;

    fn try_from(raw: String) -> Result<Self, Self::Error> {
        match raw.chars().position(|c| c == '=') {
            Some(0) => {
                let mut cmd = RawArgs::command();
                Err(cmd.error(ClapErrorKind::InvalidValue, "need argument name"))
            }
            Some(loc) => Ok(Self { raw, eq_idx: loc }),
            None => {
                let mut cmd = RawArgs::command();
                Err(cmd.error(ClapErrorKind::InvalidValue, "need a value"))
            }
        }
    }
}
