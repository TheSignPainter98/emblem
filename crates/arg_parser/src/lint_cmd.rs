use crate::{input_args::InputArgs, lua_args::LuaArgs};
use clap::Parser;
use emblem_core::Linter as EmblemLinter;

/// Arguments to the lint subcommand
#[derive(Clone, Debug, Parser, PartialEq, Eq)]
#[warn(missing_docs)]
pub struct LintCmd {
    #[command(flatten)]
    #[allow(missing_docs)]
    pub input: InputArgs,

    /// Apply fixes
    #[arg(long)]
    pub fix: bool,

    #[command(flatten)]
    #[allow(missing_docs)]
    pub lua: LuaArgs,
}

impl From<&LintCmd> for EmblemLinter {
    fn from(cmd: &LintCmd) -> Self {
        Self::new(cmd.input.file.clone().into(), cmd.fix)
    }
}

#[cfg(test)]
mod test {
    use crate::{arg_path::ArgPath, Args};

    #[test]
    fn input_file() {
        assert_eq!(
            Args::try_parse_from(["em", "lint"])
                .unwrap()
                .command
                .lint()
                .unwrap()
                .input
                .file,
            ArgPath::Path("main.em".into())
        );
        assert_eq!(
            Args::try_parse_from(["em", "lint", "-"])
                .unwrap()
                .command
                .lint()
                .unwrap()
                .input
                .file,
            ArgPath::Stdio
        );
        assert_eq!(
            Args::try_parse_from(["em", "lint", "plain.txt"])
                .unwrap()
                .command
                .lint()
                .unwrap()
                .input
                .file,
            ArgPath::Path("plain.txt".into())
        );
    }

    #[test]
    fn module_args() {
        assert_eq!(
            Args::try_parse_from(["em", "lint"])
                .unwrap()
                .command
                .lint()
                .unwrap()
                .lua
                .args,
            vec![]
        );

        {
            let valid_ext_args = Args::try_parse_from(["em", "lint", "-ak=v", "-ak2=v2", "-ak3="])
                .unwrap()
                .command
                .lint()
                .unwrap()
                .lua
                .args
                .clone();
            assert_eq!(valid_ext_args.len(), 3);
            assert_eq!(valid_ext_args[0].name(), "k");
            assert_eq!(valid_ext_args[0].value(), "v");
            assert_eq!(valid_ext_args[1].name(), "k2");
            assert_eq!(valid_ext_args[1].value(), "v2");
            assert_eq!(valid_ext_args[2].name(), "k3");
            assert_eq!(valid_ext_args[2].value(), "");
        }

        assert!(Args::try_parse_from(["em", "lint", "-a=v"]).is_err());
    }
}
