use clap::Parser;

/// Arguments to the add subcommand
#[derive(Clone, Debug, Default, Parser, PartialEq, Eq)]
#[warn(missing_docs)]
pub struct AddCmd {
    /// The extension to add
    #[arg(value_name = "source")]
    pub to_add: String,

    /// Use a specific commit in the extension's history
    #[arg(long, value_name = "hash", group = "extension-version")]
    pub commit: Option<String>,

    /// Override the extension name
    #[arg(long, value_name = "name")]
    pub rename_as: Option<String>,

    /// Use version of extension at given tag
    #[arg(long, value_name = "tag-name", group = "extension-version")]
    pub tag: Option<String>,

    /// Use a specific branch in the extension's history
    #[arg(long, value_name = "name", group = "extension-version")]
    pub branch: Option<String>,
}

#[cfg(test)]
mod test {
    use crate::Args;
    use itertools::Itertools;

    #[test]
    fn to_add() {
        assert_eq!(
            "pootis",
            Args::try_parse_from(["em", "add", "pootis"])
                .unwrap()
                .command
                .add()
                .unwrap()
                .to_add,
        );
        assert!(Args::try_parse_from(["em", "add"]).is_err());
    }

    #[test]
    fn version() {
        assert_eq!(
            None,
            Args::try_parse_from(["em", "add", "pootis"])
                .unwrap()
                .command
                .add()
                .unwrap()
                .commit
        );
        assert_eq!(
            None,
            Args::try_parse_from(["em", "add", "pootis"])
                .unwrap()
                .command
                .add()
                .unwrap()
                .tag
        );
        assert_eq!(
            None,
            Args::try_parse_from(["em", "add", "pootis"])
                .unwrap()
                .command
                .add()
                .unwrap()
                .branch
        );
        assert_eq!(
            Some("deadbeef".into()),
            Args::try_parse_from(["em", "add", "pootis", "--commit", "deadbeef"])
                .unwrap()
                .command
                .add()
                .unwrap()
                .commit
        );
        assert_eq!(
            Some("v4.5.0".into()),
            Args::try_parse_from(["em", "add", "pootis", "--tag", "v4.5.0"])
                .unwrap()
                .command
                .add()
                .unwrap()
                .tag
        );
        assert_eq!(
            Some("spah-creepn-aroun-here".into()),
            Args::try_parse_from(["em", "add", "pootis", "--branch", "spah-creepn-aroun-here"])
                .unwrap()
                .command
                .add()
                .unwrap()
                .branch
        );
        let filters = [
            ["--commit", "COMMIT"],
            ["--tag", "TAG"],
            ["--branch", "BRANCH"],
        ];
        for (f1, f2) in filters
            .iter()
            .cartesian_product(filters.iter())
            .filter(|(f1, f2)| f1 != f2)
        {
            assert!(Args::try_parse_from({
                let mut args = vec!["em", "add", "pootis"];
                args.extend_from_slice(f1);
                args.extend_from_slice(f2);
                args
            })
            .is_err());
        }
    }

    #[test]
    fn rename_as() {
        assert_eq!(
            None,
            Args::try_parse_from(["em", "add", "pootis"])
                .unwrap()
                .command
                .add()
                .unwrap()
                .rename_as
        );
        assert_eq!(
            Some("nope".into()),
            Args::try_parse_from(["em", "add", "pootis", "--rename-as", "nope"])
                .unwrap()
                .command
                .add()
                .unwrap()
                .rename_as
        );
    }
}
