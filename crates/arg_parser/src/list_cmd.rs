use crate::lua_args::LuaArgs;
use clap::{Parser, ValueEnum};

/// Arguments to the list subcommand
#[derive(Clone, Debug, Parser, PartialEq, Eq)]
#[warn(missing_docs)]
pub struct ListCmd {
    /// What to list
    #[arg(value_enum, value_name = "what")]
    pub what: RequestedInfo,

    #[command(flatten)]
    #[allow(missing_docs)]
    pub lua: LuaArgs,
}

#[derive(ValueEnum, Clone, Debug, Eq, PartialEq)]
pub enum RequestedInfo {
    // InputFormats,
    // InputExtensions,
    OutputFormats,
    OutputExtensions,
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::Args;

    #[test]
    fn list_info() {
        assert_eq!(
            Args::try_parse_from(["em", "list", "output-formats"])
                .unwrap()
                .command
                .list()
                .unwrap()
                .what,
            RequestedInfo::OutputFormats
        );
        assert_eq!(
            Args::try_parse_from(["em", "list", "output-extensions"])
                .unwrap()
                .command
                .list()
                .unwrap()
                .what,
            RequestedInfo::OutputExtensions
        );
        assert!(Args::try_parse_from(["em", "list", "root-passwd"]).is_err());
    }

    #[test]
    fn module_args() {
        assert_eq!(
            Args::try_parse_from(["em", "list", "output-formats"])
                .unwrap()
                .command
                .list()
                .unwrap()
                .lua
                .args,
            vec![]
        );

        {
            let valid_ext_args =
                Args::try_parse_from(["em", "list", "output-formats", "-ak=v", "-ak2=v2", "-ak3="])
                    .unwrap()
                    .command
                    .list()
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

        assert!(Args::try_parse_from(["em", "list", "-a=v"]).is_err());
    }
}
