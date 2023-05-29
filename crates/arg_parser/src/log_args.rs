use crate::RawArgs;
use clap::{
    error::{Error as ClapError, ErrorKind as ClapErrorKind},
    ArgAction::Count,
    CommandFactory, Parser, ValueEnum,
};

#[derive(Debug)]
pub struct LogArgs {
    /// Colourise log messages
    pub colour: bool,

    /// Make warnings into errors
    pub warnings_as_errors: bool,

    /// Output verbosity
    pub verbosity: Verbosity,
}

impl TryFrom<RawLogArgs> for LogArgs {
    type Error = ClapError;

    fn try_from(raw: RawLogArgs) -> Result<Self, Self::Error> {
        let RawLogArgs {
            colour,
            warnings_as_errors,
            verbosity,
        } = raw;
        Ok(Self {
            colour: colour.into(),
            warnings_as_errors,
            verbosity: verbosity.try_into()?,
        })
    }
}

#[derive(Debug, Parser)]
pub struct RawLogArgs {
    /// Colourise log messages
    #[arg(long, value_enum, default_value_t, value_name = "when", global = true)]
    colour: ColouriseOutput,

    /// Make warnings into errors
    #[arg(short = 'E', default_value_t = false, global = true)]
    warnings_as_errors: bool,

    /// Set output verbosity
    #[arg(short, action=Count, default_value_t=0, value_name = "level", global=true)]
    verbosity: u8,
}

#[derive(ValueEnum, Copy, Clone, Debug, Default, PartialEq, Eq)]
pub enum ColouriseOutput {
    Never,
    #[default]
    Auto,
    Always,
}

impl From<ColouriseOutput> for bool {
    fn from(hint: ColouriseOutput) -> Self {
        use supports_color::Stream;

        match hint {
            ColouriseOutput::Always => true,
            ColouriseOutput::Auto => {
                if let Some(support) = supports_color::on(Stream::Stderr) {
                    support.has_basic
                } else {
                    false
                }
            }
            ColouriseOutput::Never => false,
        }
    }
}

#[derive(ValueEnum, Copy, Clone, Debug, Default, Eq, PartialEq, Ord, PartialOrd)]
pub enum Verbosity {
    /// Output errors and warnings
    #[default]
    Terse,

    /// Output more information about what's going on
    Verbose,

    /// Show debugging info (very verbose)
    Debug,
}

impl TryFrom<u8> for Verbosity {
    type Error = clap::Error;

    fn try_from(ctr: u8) -> Result<Self, Self::Error> {
        match ctr {
            0 => Ok(Verbosity::Terse),
            1 => Ok(Verbosity::Verbose),
            2 => Ok(Verbosity::Debug),
            _ => Err(RawArgs::command().error(ClapErrorKind::TooManyValues, "too verbose")),
        }
    }
}

impl From<Verbosity> for emblem_core::Verbosity {
    fn from(v: Verbosity) -> Self {
        match v {
            Verbosity::Terse => Self::Terse,
            Verbosity::Verbose => Self::Verbose,
            Verbosity::Debug => Self::Debug,
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::Args;

    #[test]
    fn colourise_output() {
        assert!(
            !Args::try_parse_from(["em", "--colour", "never"])
                .unwrap()
                .log
                .colour
        );
        assert!(
            Args::try_parse_from(["em", "--colour", "always"])
                .unwrap()
                .log
                .colour
        );

        assert!(Args::try_parse_from(["em", "--colour", "crabcakes"]).is_err());
    }

    #[test]
    fn warnings_as_errors() {
        assert!(!Args::try_parse_from(["em"]).unwrap().log.warnings_as_errors);
        assert!(
            Args::try_parse_from(["em", "-E"])
                .unwrap()
                .log
                .warnings_as_errors
        );
    }

    #[test]
    fn verbosity() {
        assert_eq!(
            {
                let empty: [&str; 0] = [];
                Args::try_parse_from(empty).unwrap().log.verbosity
            },
            Verbosity::Terse
        );
        assert_eq!(
            Args::try_parse_from(["em"]).unwrap().log.verbosity,
            Verbosity::Terse
        );
        assert_eq!(
            Args::try_parse_from(["em", "-v"]).unwrap().log.verbosity,
            Verbosity::Verbose
        );
        assert_eq!(
            Args::try_parse_from(["em", "-vv"]).unwrap().log.verbosity,
            Verbosity::Debug
        );
        assert!(Args::try_parse_from(["em", "-vvv"]).is_err());
    }
}
