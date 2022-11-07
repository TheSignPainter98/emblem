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
    Args::command().debug_assert()
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

    /// Restrict memory available to the extension environment
    #[arg(long, value_parser = MemoryLimit::parser(), value_name = "amount")]
    pub max_mem: Option<MemoryLimit>,

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
    #[arg(long, env = "EM_STYLE_PATH", value_parser = SearchPath::parser(), value_name = "path")]
    pub style_path: Option<SearchPath>,

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
    #[arg(long, env = "EM_EXT_PATH", value_parser = SearchPath::parser(), value_name = "ext")]
    pub extension_path: Option<SearchPath>,
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
    fn colourise_output() {
        assert_eq!(Args::from(&["em"]).colour, ColouriseOutput::Auto);
        assert_eq!(
            Args::from(&["em", "--colour", "never"]).colour,
            ColouriseOutput::Never
        );
        assert_eq!(
            Args::from(&["em", "--colour", "auto"]).colour,
            ColouriseOutput::Auto
        );
        assert_eq!(
            Args::from(&["em", "--colour", "always"]).colour,
            ColouriseOutput::Always
        );
    }

    #[test]
    fn fatal_warnings() {
        assert!(!Args::from(&["em"]).fatal_warnings);
        assert!(Args::from(&["em", "-E"]).fatal_warnings);
    }

    #[test]
    fn input_driver() {
        assert_eq!(Args::from(&["em"]).input_driver, None);
        assert_eq!(
            Args::from(&["em", "-i", "chickens"]).input_driver,
            Some("chickens".to_owned())
        );
    }

    #[test]
    fn output_driver() {
        assert_eq!(Args::from(&["em"]).output_driver, None);
        assert_eq!(
            Args::from(&["em", "-o", "pies"]).output_driver,
            Some("pies".to_owned())
        );
    }

    #[test]
    fn input_file() {
        assert_eq!(Args::from(&["em"]).input_file, None);
        assert_eq!(
            Args::from(&["em", "chickens"]).input_file,
            Some("chickens".to_owned())
        );
    }

    #[test]
    fn output_file() {
        assert_eq!(Args::from(&["em"]).output_file, None);
        assert_eq!(
            Args::from(&["em", "_", "pies"]).output_file,
            Some("pies".to_owned())
        );
    }

    #[test]
    fn max_mem() {
        assert_eq!(Args::from(&["em"]).max_mem, None);
        assert_eq!(
            Args::from(&["em", "--max-mem", "25"]).max_mem,
            Some(MemoryLimit(25))
        );
        assert_eq!(
            Args::from(&["em", "--max-mem", "25K"]).max_mem,
            Some(MemoryLimit(25 * 1024))
        );
        assert_eq!(
            Args::from(&["em", "--max-mem", "25M"]).max_mem,
            Some(MemoryLimit(25 * 1024 * 1024))
        );
        assert_eq!(
            Args::from(&["em", "--max-mem", "25G"]).max_mem,
            Some(MemoryLimit(25 * 1024 * 1024 * 1024))
        );
    }

    #[test]
    fn style() {
        assert_eq!(Args::from(&["em"]).style, None);
        assert_eq!(
            Args::from(&["em", "-s", "funk"]).style,
            Some("funk".to_owned())
        );
    }

    #[test]
    fn sandbox() {
        assert_eq!(Args::from(&["em"]).sandbox, SandboxLevel::Standard);
        assert_eq!(
            Args::from(&["em", "--sandbox", "unrestricted"]).sandbox,
            SandboxLevel::Unrestricted
        );
        assert_eq!(
            Args::from(&["em", "--sandbox", "standard"]).sandbox,
            SandboxLevel::Standard
        );
        assert_eq!(
            Args::from(&["em", "--sandbox", "strict"]).sandbox,
            SandboxLevel::Strict
        );
    }

    #[test]
    fn style_path() {
        assert_eq!(Args::from(&["em"]).style_path, None);
        assert_eq!(
            Args::from(&["em", "--style-path", "club:house"]).style_path,
            Some(SearchPath::from(vec![
                "club".to_owned(),
                "house".to_owned()
            ]))
        );
    }

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

    #[test]
    fn extensions() {
        let empty: [&str; 0] = [];
        assert_eq!(Args::from(["em"]).extensions, empty);
        assert_eq!(
            Args::from(["em", "-x", "foo", "-x", "bar", "-x", "baz"]).extensions,
            ["foo".to_owned(), "bar".to_owned(), "baz".to_owned()]
        );
    }

    #[test]
    fn extension_args() {
        assert_eq!(Args::from(&["em"]).extension_args, vec![]);

        {
            let valid_ext_args = Args::from(&["em", "-ak=v", "-ak2=v2", "-ak3="]).extension_args;
            assert_eq!(valid_ext_args.len(), 3);
            assert_eq!(valid_ext_args[0].name(), "k");
            assert_eq!(valid_ext_args[0].value(), "v");
            assert_eq!(valid_ext_args[1].name(), "k2");
            assert_eq!(valid_ext_args[1].value(), "v2");
            assert_eq!(valid_ext_args[2].name(), "k3");
            assert_eq!(valid_ext_args[2].value(), "");
        }
    }

    #[test]
    fn extension_path() {
        assert_eq!(Args::from(&["em"]).extension_path, None);
        assert_eq!(
            Args::from(&["em", "--extension-path", "club:house"]).extension_path,
            Some(SearchPath::from(vec![
                "club".to_owned(),
                "house".to_owned()
            ]))
        );
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct MemoryLimit(usize);

impl MemoryLimit {
    fn parser() -> impl TypedValueParser {
        StringValueParser::new().try_map(Self::try_from)
    }
}

impl TryFrom<String> for MemoryLimit {
    type Error = error::Error;
    fn try_from(raw: String) -> Result<Self, Self::Error> {
        if raw.len() == 0 {
            let mut cmd = Args::command();
            return Err(cmd.error(error::ErrorKind::InvalidValue, "need amount"));
        }

        let (raw_amt, unit): (String, String) = raw.chars().partition(|c| c.is_numeric());

        let amt: usize = match raw_amt.parse() {
            Ok(a) => a,
            Err(e) => {
                let mut cmd = Args::command();
                return Err(cmd.error(error::ErrorKind::InvalidValue, e));
            }
        };

        let multiplier: usize = {
            match &unit[..] {
                "K" => 1 << 10,
                "M" => 1 << 20,
                "G" => 1 << 30,
                "" => 1,
                _ => {
                    let mut cmd = Args::command();
                    return Err(cmd.error(
                        error::ErrorKind::InvalidValue,
                        format!("unrecognised unit: {}", unit),
                    ));
                }
            }
        };

        Ok(Self(amt * multiplier))
    }
}

#[cfg(test)]
mod test_memory_limit {
    use super::*;

    #[test]
    fn try_from() {
        assert!(MemoryLimit::try_from("".to_owned()).is_err());

        assert_eq!(
            MemoryLimit::try_from("10".to_owned()).unwrap(),
            MemoryLimit(10)
        );
        assert_eq!(
            MemoryLimit::try_from("10K".to_owned()).unwrap(),
            MemoryLimit(10 * 1024)
        );
        assert_eq!(
            MemoryLimit::try_from("10M".to_owned()).unwrap(),
            MemoryLimit(10 * 1024 * 1024)
        );
        assert_eq!(
            MemoryLimit::try_from("10G".to_owned()).unwrap(),
            MemoryLimit(10 * 1024 * 1024 * 1024)
        );

        assert!(MemoryLimit::try_from("10Q".to_owned()).is_err());
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SearchPath {
    path: Vec<String>,
}

impl SearchPath {
    fn parser() -> impl TypedValueParser {
        StringValueParser::new().map(Self::from)
    }

    // pub fn search(&self, target: &str) -> Result<&str, error::Error> {
    //  TODO(kcza): complete me!
    // }

    // pub fn open(&self, target: &str, cwd: Option<&str>) -> io::File {
    //  TODO(kcza): complete me!
    // }
}

impl Default for SearchPath {
    fn default() -> Self {
        Self { path: vec![] }
    }
}

impl From<String> for SearchPath {
    fn from(raw: String) -> Self {
        Self::from(&raw[..])
    }
}

impl From<&str> for SearchPath {
    fn from(raw: &str) -> Self {
        Self {
            path: raw
                .split(':')
                .filter(|s| s.len() != 0)
                .map(|s| s.to_owned())
                .collect(),
        }
    }
}

impl From<Vec<String>> for SearchPath {
    fn from(path: Vec<String>) -> Self {
        Self { path }
    }
}

#[cfg(test)]
mod test_search_path {
    use super::*;

    #[test]
    fn from() {
        assert_eq!(
            SearchPath::from("foo:bar::baz"),
            SearchPath {
                path: vec!["foo".to_owned(), "bar".to_owned(), "baz".to_owned()],
            }
        );

        assert_eq!(
            SearchPath::from("foo:bar::baz".to_owned()),
            SearchPath {
                path: vec!["foo".to_owned(), "bar".to_owned(), "baz".to_owned()],
            }
        );
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

#[derive(ValueEnum, Clone, Debug, Default, PartialEq, Eq)]
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

#[derive(ValueEnum, Clone, Debug, Default, PartialEq, Eq)]
pub enum ColouriseOutput {
    Never,
    #[default]
    Auto,
    Always,
}

/// Command-line arg declaration
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ExtArg {
    raw: String,
    loc: usize,
}

impl ExtArg {
    pub fn parser() -> impl TypedValueParser {
        StringValueParser::new().try_map(Self::try_parse)
    }

    fn try_parse(raw: String) -> Result<Self, error::Error> {
        match raw.chars().position(|c| c == '=') {
            Some(0) => {
                let mut cmd = Args::command();
                Err(cmd.error(error::ErrorKind::InvalidValue, "need argument name"))
            }
            Some(loc) => Ok(Self { raw, loc }),
            None => {
                let mut cmd = Args::command();
                Err(cmd.error(error::ErrorKind::InvalidValue, "need a value"))
            }
        }
    }

    pub fn name(&self) -> &str {
        &self.raw[..self.loc]
    }

    pub fn value(&self) -> &str {
        &self.raw[self.loc + 1..]
    }
}
