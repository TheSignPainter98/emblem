use crate::input_args::InputArgs;
use clap::Parser;

/// Arguments to the fmt subcommand
#[derive(Clone, Debug, Parser, PartialEq, Eq)]
#[warn(missing_docs)]
pub struct FormatCmd {
    #[command(flatten)]
    #[allow(missing_docs)]
    pub input: InputArgs,
}

#[cfg(test)]
mod test {
    use crate::{arg_path::ArgPath, Args};

    #[test]
    fn input_file() {
        assert_eq!(
            Args::try_parse_from(["em", "fmt"])
                .unwrap()
                .command
                .format()
                .unwrap()
                .input
                .file,
            ArgPath::Path("main.em".into())
        );
        assert_eq!(
            Args::try_parse_from(["em", "fmt", "-"])
                .unwrap()
                .command
                .format()
                .unwrap()
                .input
                .file,
            ArgPath::Stdio
        );
        assert_eq!(
            Args::try_parse_from(["em", "fmt", "plain.txt"])
                .unwrap()
                .command
                .format()
                .unwrap()
                .input
                .file,
            ArgPath::Path("plain.txt".into())
        );
    }
}
