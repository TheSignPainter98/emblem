use clap::{
    builder::{StringValueParser, TypedValueParser},
    error,
    ArgAction::{Append, Count, Help, Version},
    CommandFactory, Parser, ValueEnum,
    ValueHint::{AnyPath, FilePath},
};
use num_enum::FromPrimitive;

const LONG_ABOUT: &str = "Takes input of a markdown-like document, processes it and typesets it before passing the result to a driver for outputting in some format. Extensions can be used to include arbitrary functionality; device drivers are also extensions.";

pub fn parse() -> Args {
    let mut args = Args::parse();

    if args.verbosity_ctr >= 3 {
        let mut cmd = Args::command();
        let err = cmd.error(error::ErrorKind::TooManyValues, "too verbose");
        err.exit();
    }
    if let Ok(v) = Verbosity::try_from(args.verbosity_ctr) {
        args.verbosity = v;
    }

    args
}

#[derive(Parser, Debug)]
#[command(author, version, about, long_about=LONG_ABOUT, disable_help_flag=true, disable_version_flag=true)]
pub struct Args {
    /// Pass variable into extension-space
    #[arg(short = 'a', action = Append, value_parser = StringValueParser::new().try_map(ExtArg::try_parse),  value_name="var=value")]
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

#[derive(ValueEnum, Clone, Debug)]
pub enum RequestedInfo {
    InputFormats,
    InputExtensions,
    OutputFormats,
    OutputExtensions,
}

#[derive(ValueEnum, Clone, Debug, Default, FromPrimitive)]
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
