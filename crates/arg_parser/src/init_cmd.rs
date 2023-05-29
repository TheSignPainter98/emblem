use clap::{Parser, ValueHint::DirPath};

/// Arguments to the init subcommand
#[derive(Clone, Debug, Parser, PartialEq, Eq)]
#[warn(missing_docs)]
pub struct InitCmd {
    /// Directory to contain the new document
    #[arg(value_name = "dir", value_hint = DirPath, default_value = ".")]
    pub dir: String,
}

#[cfg(test)]
mod test {
    use crate::Args;

    #[test]
    fn dir() {
        assert_eq!(
            Args::try_parse_from(["em", "init"])
                .unwrap()
                .command
                .init()
                .unwrap()
                .dir,
            ".",
        );
        assert_eq!(
            Args::try_parse_from(["em", "init", "cool-doc"])
                .unwrap()
                .command
                .init()
                .unwrap()
                .dir,
            "cool-doc",
        );
    }
}
