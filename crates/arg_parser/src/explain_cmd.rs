use clap::Parser;
use emblem_core::Explainer as EmblemExplainer;

/// Arguments to the explain subcommand
#[derive(Clone, Debug, Parser, PartialEq, Eq)]
#[warn(missing_docs)]
pub struct ExplainCmd {
    /// Code of the error to explain
    #[arg(value_name = "error-code")]
    pub id: String,
}

impl From<&ExplainCmd> for EmblemExplainer {
    fn from(cmd: &ExplainCmd) -> Self {
        Self::new(cmd.id.clone())
    }
}

#[cfg(test)]
mod test {
    use crate::Args;

    #[test]
    fn code() {
        assert_eq!(
            Args::try_parse_from(["em", "explain", "E001"])
                .unwrap()
                .command
                .explain()
                .unwrap()
                .id,
            "E001"
        );
        assert!(Args::try_parse_from(["em", "explain"]).is_err());
    }
}
