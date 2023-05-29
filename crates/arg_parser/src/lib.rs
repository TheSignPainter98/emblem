mod add_cmd;
mod arg_path;
mod build_cmd;
mod command;
mod explain_cmd;
mod ext_arg;
mod format_cmd;
mod init_cmd;
mod input_args;
mod lint_cmd;
mod list_cmd;
mod log_args;
mod lua_args;
mod output_args;
mod resource_limit;
mod sandbox_level;

pub use crate::add_cmd::AddCmd;
pub use crate::build_cmd::BuildCmd;
pub use crate::explain_cmd::ExplainCmd;
pub use crate::format_cmd::FormatCmd;
pub use crate::init_cmd::InitCmd;
pub use crate::lint_cmd::LintCmd;
pub use crate::list_cmd::ListCmd;
pub use command::Command;
pub use input_args::InputArgs;
pub use log_args::LogArgs;
pub use lua_args::LuaArgs;
pub use output_args::OutputArgs;

use crate::log_args::RawLogArgs;
use clap::{
    ArgAction::{Help, Version},
    Parser,
};
use std::{env, ffi::OsString};

/// Parsed command-line arguments
#[derive(Debug)]
#[warn(missing_docs)]
pub struct Args {
    /// Action to take
    pub command: Command,

    /// Logger arguments
    pub log: LogArgs,
}

impl Args {
    /// Parse command-line arguments, exit on failure
    pub fn parse() -> Self {
        match Self::try_parse_from(env::args()) {
            Ok(args) => args,
            Err(e) => e.exit(),
        }
    }

    pub fn try_parse_from<I, T>(iter: I) -> Result<Self, clap::Error>
    where
        T: Into<OsString> + Clone,
        I: IntoIterator<Item = T>,
    {
        Self::try_from(RawArgs::try_parse_from(iter)?)
    }
}

impl Args {
    pub fn lua_args(&self) -> Option<&LuaArgs> {
        self.command.lua_args()
    }
}

impl TryFrom<RawArgs> for Args {
    type Error = clap::Error;

    fn try_from(raw: RawArgs) -> Result<Self, Self::Error> {
        let RawArgs {
            command,
            log,
            help: _,
            version: _,
        } = raw;

        let command = command.unwrap_or_default();
        let log = log.try_into()?;

        Ok(Self { command, log })
    }
}

const LONG_ABOUT: &str = "Takes input of a markdown-like document, processes it and typesets it before passing the result to a driver for outputting in some format. Extensions can be used to include arbitrary functionality; device drivers can be defined by extensions.";

/// Internal command-line argument parser
#[derive(Parser, Debug)]
#[command(name="em", author, version, about, long_about=LONG_ABOUT, disable_help_flag=true, disable_version_flag=true)]
#[warn(missing_docs)]
pub struct RawArgs {
    #[command(subcommand)]
    command: Option<Command>,

    #[command(flatten)]
    #[allow(missing_docs)]
    pub log: RawLogArgs,

    /// Print help information, use `--help` for more detail
    #[arg(short, long, action=Help, global=true)]
    help: Option<bool>,

    /// Print version info
    #[arg(long, action=Version)]
    version: Option<bool>,
}

#[cfg(test)]
mod test {
    use super::*;
    use clap::CommandFactory;

    #[test]
    fn debug_assert() {
        RawArgs::command().debug_assert()
    }

    #[test]
    fn default() {
        assert_eq!(
            Args::try_parse_from(["em"]).unwrap().command,
            Args::try_parse_from(["em", "build"]).unwrap().command
        );
    }
}
