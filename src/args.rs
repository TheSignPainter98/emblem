use clap::{
    builder::{StringValueParser, TypedValueParser},
    error,
    ArgAction::{Append, Count, Help, Version},
    CommandFactory, Parser, ValueEnum,
    ValueHint::{AnyPath, FilePath},
};
use num_enum::FromPrimitive;
use std::ffi::OsString;

const LONG_ABOUT: &str = "Takes input of a markdown-like document, processes it and typesets it before passing the result to a driver for outputting in some format. Extensions can be used to include arbitrary functionality; device drivers are also extensions.";

#[test]
fn test_cmd() {
    Args::command().debug_assert();
}

/// Parsed command-line arguments
#[derive(Parser, Debug)]
#[command(author, version, about, long_about=LONG_ABOUT, disable_help_flag=true, disable_version_flag=true)]
#[warn(missing_docs)]
pub struct Args {
    /// Pass variable into extension-space
    #[arg(short = 'a', action = Append, value_parser = ExtArg::parser(),  value_name="arg=value")]
    pub extension_args: Vec<ExtArg>,

    /// Colourise log messages
    #[arg(long, value_enum, default_value_t, value_name = "when")]
    pub colour: ColouriseOutput,

    /// Make warnings fatal
    #[arg(short = 'E', default_value_t = false)]
    pub fatal_warnings: bool,

    /// Override detected input format
    #[arg(short, value_name = "format")]
    pub input_driver: Option<String>,

    /// File to typeset
    #[arg(value_name = "in-file", value_hint=FilePath)]
    pub input_file: Option<String>,

    /// Print help information, use `--help` for more detail
    #[arg(short, long, action=Help)]
    help: Option<bool>,

    /// Print info and exit
    #[arg(long = "list", value_enum, value_name = "what")]
    pub list_info: Option<RequestedInfo>,

    /// Override detected output format
    #[arg(short, value_name = "format")]
    pub output_driver: Option<String>,

    /// Output file path
    #[arg(value_name = "out-file", value_hint=AnyPath)]
    pub output_file: Option<String>,

    /// Set root stylesheet
    #[arg(short, value_name = "style")]
    pub style: Option<String>,

    /// Restrict system access
    #[arg(long, value_enum, default_value_t, value_name = "level")]
    pub sandbox: SandboxLevel,

    /// Style search-path, colon-separated
    #[arg(long, env = "EM_STYLE_PATH", value_name = "path")]
    pub style_path: Option<String>,

    /// Set output verbosity
    #[arg(short, action=Count, default_value_t=0, value_name = "level")]
    verbosity_ctr: u8,

    /// Parsed output verbosity
    #[clap(skip)]
    pub verbosity: Verbosity,

    /// Print version info
    #[arg(long, action=Version)]
    version: Option<bool>,

    /// Load an extension
    #[arg(short = 'x', action=Append, value_name = "ext")]
    pub extensions: Vec<String>,

    /// Extension search-path, colon-separated
    #[arg(long, action=Append, value_name = "ext")]
    pub extension_path: Vec<String>,
}

impl Args {
    /// Parse command-line arguments
    pub fn new() -> Self {
        Args::parse().sanitised()
    }

    /// Validate and infer argument values
    fn sanitised(mut self) -> Self {
        if self.verbosity_ctr >= 3 {
            let mut cmd = Args::command();
            let err = cmd.error(error::ErrorKind::TooManyValues, "too verbose");
            err.exit();
        }
        if let Ok(v) = Verbosity::try_from(self.verbosity_ctr) {
            self.verbosity = v;
        }
        self
    }
}

impl<T, I> From<I> for Args
where
    T: Into<OsString> + Clone,
    I: IntoIterator<Item = T>,
{
    fn from(itr: I) -> Self {
        Args::parse_from(itr).sanitised()
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn verbosity() {
        assert_eq!(
            {
                let empty: [&str; 0] = [];
                Args::from(empty).verbosity
            },
            Verbosity::Terse
        );
        assert_eq!(Args::from(["em"]).verbosity, Verbosity::Terse);
        assert_eq!(Args::from(["em", "-v"]).verbosity, Verbosity::Verbose);
        assert_eq!(Args::from(["em", "-vv"]).verbosity, Verbosity::Debug);
    }
}

#[derive(ValueEnum, Clone, Debug)]
pub enum RequestedInfo {
    InputFormats,
    InputExtensions,
    OutputFormats,
    OutputExtensions,
}

#[derive(ValueEnum, Clone, Debug, Default, FromPrimitive, Eq, PartialEq)]
#[repr(u8)]
pub enum Verbosity {
    /// Output errors and warnings
    #[default]
    Terse,

    /// Output more information about what's going on
    Verbose,

    /// Show debugging info (very verbose)
    Debug,
}

#[derive(ValueEnum, Clone, Debug, Default)]
pub enum SandboxLevel {
    /// Extensions have no restrictions placed upon them.
    Unrestricted,

    /// Prohibit creation of new subprocesses and file system access outside of the current
    /// working directory.
    #[default]
    Standard,

    /// Same restrictions as Standard, but all file system access if prohibited.
    Strict,
}

#[derive(ValueEnum, Clone, Debug, Default)]
pub enum ColouriseOutput {
    Never,
    #[default]
    Auto,
    Always,
}

/// Command-line arg declaration
#[derive(Clone, Debug)]
pub struct ExtArg {
    /// Name of the variable to assign
    pub name: String,

    /// Value to pass to the given variable
    pub value: String,
}

impl ExtArg {
    pub fn parser() -> impl TypedValueParser {
        StringValueParser::new().try_map(ExtArg::try_parse)
    }

    fn try_parse(raw: String) -> Result<Self, error::Error> {
        let raw = raw.to_owned();
        match raw.chars().position(|c| c == '=') {
            Some(0) => {
                let mut cmd = Args::command();
                Err(cmd.error(error::ErrorKind::InvalidValue, "var name must not be empty"))
            }
            Some(loc) => Ok(Self {
                name: raw[..loc].into(),
                value: raw[loc + 1..].into(),
            }),
            None => {
                let mut cmd = Args::command();
                Err(cmd.error(error::ErrorKind::InvalidValue, "need '=' in variable def"))
            }
        }
    }
}
