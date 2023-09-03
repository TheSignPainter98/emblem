use clap::{
    builder::{StringValueParser, TypedValueParser},
    error::{Error as ClapError, ErrorKind as ClapErrorKind},
    CommandFactory, Parser,
};
use emblem_core::{log::LogId as EmblemLogId, Explainer as EmblemExplainer};

use crate::RawArgs;

/// Arguments to the explain subcommand
#[derive(Clone, Debug, Parser, PartialEq, Eq)]
#[warn(missing_docs)]
pub struct ExplainCmd {
    /// Code of the error to explain
    #[arg(value_name = "error-code", value_parser = LogId::parser())]
    pub id: LogId,
}

impl From<&ExplainCmd> for EmblemExplainer {
    fn from(cmd: &ExplainCmd) -> Self {
        Self::new(cmd.id.clone().into())
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct LogId(String);

impl LogId {
    fn parser() -> impl TypedValueParser {
        StringValueParser::new().try_map(Self::try_from)
    }
}

impl TryFrom<String> for LogId {
    type Error = ClapError;

    fn try_from(raw: String) -> Result<Self, Self::Error> {
        if raw.is_empty() {
            return Err(
                RawArgs::command().error(ClapErrorKind::InvalidValue, "error-code cannot be empty")
            );
        }

        Ok(LogId(raw))
    }
}

impl TryFrom<&'static str> for LogId {
    type Error = ClapError;

    fn try_from(raw: &'static str) -> Result<Self, Self::Error> {
        Self::try_from(raw.to_string())
    }
}

impl From<LogId> for EmblemLogId {
    fn from(raw: LogId) -> Self {
        raw.0.into()
    }
}

#[cfg(test)]
mod test {
    use crate::{explain_cmd::LogId, Args};

    #[test]
    fn code() {
        assert_eq!(
            Args::try_parse_from(["em", "explain", "E001"])
                .unwrap()
                .command
                .explain()
                .unwrap()
                .id,
            LogId::try_from("E001").unwrap(),
        );
        assert!(Args::try_parse_from(["em", "explain", ""]).is_err());
        assert!(Args::try_parse_from(["em", "explain"]).is_err());
    }
}
