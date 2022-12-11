use clap::{
    builder::{OsStr, StringValueParser, TypedValueParser},
    error,
    ArgAction::{Append, Count, Help, Version},
    CommandFactory, Parser, ValueEnum,
    ValueHint::{AnyPath, FilePath},
};
use derive_new::new;
use std::ffi::OsString;
use std::fmt::Display;
use std::{env, fs, io, path};

/// Parsed command-line arguments
#[derive(Debug)]
pub struct Args {
    /// Pass variable into extension-space
    pub extension_args: Vec<ExtArg>,

    /// Colourise log messages
    pub colour: ColouriseOutput,

    /// Make warnings fatal
    pub fatal_warnings: bool,

    /// Override detected input format
    pub input_driver: Option<String>,

    /// File to typeset
    pub input_file: ArgPath,

    /// Print info and exit
    pub list_info: Option<RequestedInfo>,

    /// Limit lua memory usage
    pub max_mem: MemoryLimit,

    /// Override detected output format
    pub output_driver: Option<String>,

    /// Output file path
    pub output_stem: ArgPath,

    /// Set root stylesheet
    pub style: Option<String>,

    /// Restrict system access
    pub sandbox: SandboxLevel,

    /// Style search-path, colon-separated
    pub style_path: SearchPath,

    /// Output verbosity
    pub verbosity: Verbosity,

    /// Load an extension
    pub extensions: Vec<String>,

    /// Extension search-path, colon-separated
    pub extension_path: SearchPath,
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

impl TryFrom<RawArgs> for Args {
    type Error = clap::Error;

    fn try_from(raw: RawArgs) -> Result<Self, Self::Error> {
        let RawArgs {
            extension_args,
            colour,
            fatal_warnings,
            input_driver,
            input_file,
            help: _,
            list_info,
            max_mem,
            output_driver,
            output_stem,
            style,
            sandbox,
            style_path,
            verbosity,
            version: _,
            extensions,
            extension_path,
        } = raw;

        let input_file = input_file.infer_input();
        let output_stem = output_stem.infer_output(&input_file);
        let verbosity = verbosity.try_into()?;

        Ok(Self {
            extension_args,
            colour,
            fatal_warnings,
            input_driver,
            input_file,
            list_info,
            max_mem,
            output_driver,
            output_stem,
            style,
            sandbox,
            style_path,
            verbosity,
            extensions,
            extension_path,
        })
    }
}

const LONG_ABOUT: &str = "Takes input of a markdown-like document, processes it and typesets it before passing the result to a driver for outputting in some format. Extensions can be used to include arbitrary functionality; device drivers are also extensions.";

/// Internal command-line argument parser
#[derive(Parser, Debug)]
#[command(author, version, about, long_about=LONG_ABOUT, disable_help_flag=true, disable_version_flag=true)]
#[warn(missing_docs)]
struct RawArgs {
    /// Pass variable into extension-space
    #[arg(short = 'a', action = Append, value_parser = ExtArg::parser(),  value_name="arg=value")]
    extension_args: Vec<ExtArg>,

    /// Colourise log messages
    #[arg(long, value_enum, default_value_t, value_name = "when")]
    colour: ColouriseOutput,

    /// Make warnings fatal
    #[arg(short = 'E', default_value_t = false)]
    fatal_warnings: bool,

    /// Override detected input format
    #[arg(short, value_name = "format")]
    input_driver: Option<String>,

    /// Print help information, use `--help` for more detail
    #[arg(short, long, action=Help)]
    help: Option<bool>,

    /// File to typeset
    #[arg(value_name = "in-file", value_hint = FilePath, default_value_t = UninferredArgPath::default(), value_parser = UninferredArgPath::parser())]
    input_file: UninferredArgPath,

    /// Print info and exit
    #[arg(long = "list", value_enum, value_name = "what")]
    list_info: Option<RequestedInfo>,

    /// Limit lua memory usage
    #[arg(long, value_parser = MemoryLimit::parser(), default_value = "unlimited", value_name = "amount")]
    max_mem: MemoryLimit,

    /// Override detected output format
    #[arg(short, value_name = "format")]
    output_driver: Option<String>,

    /// Output file path
    #[arg(value_name = "out-file", value_hint = AnyPath, default_value_t=UninferredArgPath::default(), value_parser = UninferredArgPath::parser())]
    output_stem: UninferredArgPath,

    /// Set root stylesheet
    #[arg(short, value_name = "style")]
    style: Option<String>,

    /// Restrict system access
    #[arg(long, value_enum, default_value_t, value_name = "level")]
    sandbox: SandboxLevel,

    /// Style search-path, colon-separated
    #[arg(long, env = "EM_STYLE_PATH", value_parser = SearchPath::parser(), default_value = "", value_name = "path")]
    style_path: SearchPath,

    /// Set output verbosity
    #[arg(short, action=Count, default_value_t=0, value_name = "level")]
    verbosity: u8,

    /// Print version info
    #[arg(long, action=Version)]
    version: Option<bool>,

    /// Load an extension
    #[arg(short = 'x', action=Append, value_name = "ext")]
    extensions: Vec<String>,

    /// Extension search-path, colon-separated
    #[arg(long, env = "EM_EXT_PATH", value_parser = SearchPath::parser(), default_value = "", value_name = "path")]
    extension_path: SearchPath,
}

#[derive(Clone, Default, Debug, PartialEq, Eq)]
enum UninferredArgPath {
    #[default]
    Infer,
    Stdio,
    Path(path::PathBuf),
}

impl UninferredArgPath {
    fn parser() -> impl TypedValueParser {
        StringValueParser::new().try_map(Self::try_from)
    }

    fn infer_input(&self) -> ArgPath {
        match self {
            Self::Infer => ArgPath::Path(path::PathBuf::from("main")),
            Self::Stdio => ArgPath::Stdio,
            Self::Path(p) => ArgPath::Path(p.clone()),
        }
    }

    fn infer_output(&self, input: &ArgPath) -> ArgPath {
        match self {
            Self::Infer => match input {
                ArgPath::Stdio => ArgPath::Stdio,
                ArgPath::Path(s) => ArgPath::Path(s.clone()),
            },
            Self::Stdio => ArgPath::Stdio,
            Self::Path(s) => ArgPath::Path(s.clone()),
        }
    }
}

impl Display for UninferredArgPath {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let repr = match self {
            Self::Infer => "??",
            Self::Stdio => "stdio",
            Self::Path(p) => p.to_str().unwrap(),
        };
        repr.fmt(f)
    }
}

impl TryFrom<OsStr> for UninferredArgPath {
    type Error = error::Error;

    fn try_from(raw: OsStr) -> Result<Self, Self::Error> {
        if let Some(s) = raw.to_str() {
            return Self::try_from(s);
        }
        Err(RawArgs::command().error(
            error::ErrorKind::InvalidValue,
            format!("could not convert '{:?}' to an OS string", raw),
        ))
    }
}

impl TryFrom<String> for UninferredArgPath {
    type Error = error::Error;

    fn try_from(raw: String) -> Result<Self, Self::Error> {
        Self::try_from(&raw[..])
    }
}

impl TryFrom<&str> for UninferredArgPath {
    type Error = error::Error;

    fn try_from(raw: &str) -> Result<Self, Self::Error> {
        match raw {
            "" => Err(RawArgs::command().error(
                error::ErrorKind::InvalidValue,
                "file path cannot be empty",
            )),
            "-" => Ok(Self::Stdio),
            "??" => Ok(Self::Infer),
            path => Ok(Self::Path(path.into())),
        }
    }
}

#[derive(Debug, Default, PartialEq, Eq)]
pub enum ArgPath {
    #[default]
    Stdio,
    Path(path::PathBuf),
}

impl From<&str> for ArgPath {
    fn from(raw: &str) -> Self {
        Self::Path(path::PathBuf::from(raw))
    }
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub enum MemoryLimit {
    Limited(usize),
    #[default]
    Unlimited,
}

impl MemoryLimit {
    fn parser() -> impl TypedValueParser {
        StringValueParser::new().try_map(Self::try_from)
    }
}

impl TryFrom<OsStr> for MemoryLimit {
    type Error = error::Error;

    fn try_from(raw: OsStr) -> Result<Self, Self::Error> {
        if let Some(s) = raw.to_str() {
            return Self::try_from(s);
        }

        let mut cmd = RawArgs::command();
        Err(cmd.error(
            error::ErrorKind::InvalidValue,
            format!("could not convert '{:?}' to an OS string", raw),
        ))
    }
}

impl TryFrom<String> for MemoryLimit {
    type Error = error::Error;

    fn try_from(raw: String) -> Result<Self, Self::Error> {
        Self::try_from(&raw[..])
    }
}

impl TryFrom<&str> for MemoryLimit {
    type Error = error::Error;

    fn try_from(raw: &str) -> Result<Self, Self::Error> {
        if raw.is_empty() {
            let mut cmd = RawArgs::command();
            return Err(cmd.error(error::ErrorKind::InvalidValue, "need amount"));
        }

        if raw == "unlimited" {
            return Ok(Self::Unlimited);
        }

        let (raw_amt, unit): (String, String) = raw.chars().partition(|c| c.is_numeric());

        let amt: usize = match raw_amt.parse() {
            Ok(a) => a,
            Err(e) => {
                let mut cmd = RawArgs::command();
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
                    let mut cmd = RawArgs::command();
                    return Err(cmd.error(
                        error::ErrorKind::InvalidValue,
                        format!("unrecognised unit: {}", unit),
                    ));
                }
            }
        };

        Ok(Self::Limited(amt * multiplier))
    }
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct SearchPath {
    path: Vec<path::PathBuf>,
}

impl SearchPath {
    fn parser() -> impl TypedValueParser {
        StringValueParser::new().map(Self::from)
    }

    pub fn open<S, T>(&self, src: S, target: T) -> Result<SearchResult, io::Error>
    where
        S: Into<path::PathBuf>,
        T: AsRef<path::Path>,
    {
        let target = target.as_ref();

        if target.is_absolute() {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                format!("Absolute paths are forbidden: got {:?}", target,),
            ));
        }

        let src = src.into().canonicalize()?;

        let localpath = path::PathBuf::from(&src).join(target);
        if localpath.starts_with(&src) {
            if let Ok(f) = fs::File::open(&localpath) {
                if let Ok(metadata) = f.metadata() {
                    if metadata.is_file() {
                        return Ok(SearchResult::new(localpath, f));
                    }
                }
            }
        }

        for dir in self.normalised().path {
            let needle = {
                let p = path::PathBuf::from(&dir).join(target);
                match p.canonicalize() {
                    Ok(p) => p,
                    _ => continue,
                }
            };

            if !needle.starts_with(&dir) {
                continue;
            }

            if let Ok(f) = fs::File::open(&needle) {
                if let Ok(metadata) = f.metadata() {
                    if metadata.is_file() {
                        return Ok(SearchResult::new(needle, f));
                    }
                }
            }
        }

        Err(io::Error::new(
            io::ErrorKind::NotFound,
            format!(
                "Could not find file {:?} along path \"{}\"",
                target.as_os_str(),
                self.to_string()
            ),
        ))
    }

    fn normalised(&self) -> Self {
        Self {
            path: self.path.iter().flat_map(|d| d.canonicalize()).collect(),
        }
    }
}

impl ToString for SearchPath {
    fn to_string(&self) -> String {
        self.path
            .iter()
            .map(|dir| dir.to_str().unwrap_or("?"))
            .collect::<Vec<&str>>()
            .join(":")
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
                .filter(|s| !s.is_empty())
                .map(|s| s.into())
                .collect(),
        }
    }
}

impl<S> From<Vec<S>> for SearchPath
where
    S: Into<path::PathBuf>,
{
    fn from(raw: Vec<S>) -> Self {
        let mut path = vec![];
        for p in raw {
            path.push(p.into());
        }
        Self { path }
    }
}

#[derive(Debug, new)]
pub struct SearchResult {
    path: path::PathBuf,
    file: fs::File,
}

impl SearchResult {
    pub fn path(&self) -> &path::Path {
        &self.path
    }

    pub fn file(&self) -> &fs::File {
        &self.file
    }
}

#[derive(ValueEnum, Clone, Debug, Eq, PartialEq)]
pub enum RequestedInfo {
    InputFormats,
    InputExtensions,
    OutputFormats,
    OutputExtensions,
}

#[derive(ValueEnum, Clone, Debug, Default, Eq, PartialEq)]
pub enum Verbosity {
    /// Output errors and warnings
    #[default]
    Terse,

    /// Output more information about what's going on
    Verbose,

    /// Show debugging info (very verbose)
    Debug,
}

impl TryFrom<u8> for Verbosity {
    type Error = clap::Error;

    fn try_from(ctr: u8) -> Result<Self, Self::Error> {
        match ctr {
            0 => Ok(Verbosity::Terse),
            1 => Ok(Verbosity::Verbose),
            2 => Ok(Verbosity::Debug),
            _ => Err(RawArgs::command().error(error::ErrorKind::TooManyValues, "too verbose")),
        }
    }
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
    eq_idx: usize,
}

impl ExtArg {
    pub fn parser() -> impl TypedValueParser {
        StringValueParser::new().try_map(Self::try_from)
    }

    pub fn name(&self) -> &str {
        &self.raw[..self.eq_idx]
    }

    pub fn value(&self) -> &str {
        &self.raw[self.eq_idx + 1..]
    }
}

impl TryFrom<String> for ExtArg {
    type Error = error::Error;

    fn try_from(raw: String) -> Result<Self, Self::Error> {
        match raw.chars().position(|c| c == '=') {
            Some(0) => {
                let mut cmd = RawArgs::command();
                Err(cmd.error(error::ErrorKind::InvalidValue, "need argument name"))
            }
            Some(loc) => Ok(Self { raw, eq_idx: loc }),
            None => {
                let mut cmd = RawArgs::command();
                Err(cmd.error(error::ErrorKind::InvalidValue, "need a value"))
            }
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    mod args {
        use super::*;

        #[test]
        fn debug_assert() {
            RawArgs::command().debug_assert()
        }

        #[test]
        fn colourise_output() {
            assert_eq!(
                Args::try_parse_from(&["em"]).unwrap().colour,
                ColouriseOutput::Auto
            );
            assert_eq!(
                Args::try_parse_from(&["em", "--colour", "never"])
                    .unwrap()
                    .colour,
                ColouriseOutput::Never
            );
            assert_eq!(
                Args::try_parse_from(&["em", "--colour", "auto"])
                    .unwrap()
                    .colour,
                ColouriseOutput::Auto
            );
            assert_eq!(
                Args::try_parse_from(&["em", "--colour", "always"])
                    .unwrap()
                    .colour,
                ColouriseOutput::Always
            );

            assert!(Args::try_parse_from(&["em", "--colour", "crabcakes"]).is_err());
        }

        #[test]
        fn fatal_warnings() {
            assert!(!Args::try_parse_from(&["em"]).unwrap().fatal_warnings);
            assert!(Args::try_parse_from(&["em", "-E"]).unwrap().fatal_warnings);
        }

        #[test]
        fn input_driver() {
            assert_eq!(Args::try_parse_from(&["em"]).unwrap().input_driver, None);
            assert_eq!(
                Args::try_parse_from(&["em", "-i", "chickens"])
                    .unwrap()
                    .input_driver,
                Some("chickens".to_owned())
            );
        }

        #[test]
        fn output_driver() {
            assert_eq!(Args::try_parse_from(&["em"]).unwrap().output_driver, None);
            assert_eq!(
                Args::try_parse_from(&["em", "-o", "pies"])
                    .unwrap()
                    .output_driver,
                Some("pies".to_owned())
            );
        }

        #[test]
        fn input_file() {
            assert_eq!(
                Args::try_parse_from(&["em"]).unwrap().input_file,
                ArgPath::from("main")
            );
            assert_eq!(
                Args::try_parse_from(&["em", "-"]).unwrap().input_file,
                ArgPath::Stdio
            );
            assert_eq!(
                Args::try_parse_from(&["em", "chickens"])
                    .unwrap()
                    .input_file,
                ArgPath::from("chickens")
            );
        }

        #[test]
        fn output_stem() {
            assert_eq!(
                Args::try_parse_from(&["em"]).unwrap().output_stem,
                ArgPath::from("main"),
            );
            assert_eq!(
                Args::try_parse_from(&["em", "-"]).unwrap().output_stem,
                ArgPath::Stdio,
            );
            assert_eq!(
                Args::try_parse_from(&["em", "-", "pies"])
                    .unwrap()
                    .output_stem,
                ArgPath::from("pies"),
            );
            assert_eq!(
                Args::try_parse_from(&["em", "_", "-"]).unwrap().output_stem,
                ArgPath::Stdio,
            );
            assert_eq!(
                Args::try_parse_from(&["em", "_", "pies"])
                    .unwrap()
                    .output_stem,
                ArgPath::from("pies")
            );
            assert_eq!(
                Args::try_parse_from(&["em", "-", "pies"])
                    .unwrap()
                    .output_stem,
                ArgPath::from("pies")
            );
        }

        #[test]
        fn list_info() {
            assert_eq!(Args::try_parse_from(&["em"]).unwrap().list_info, None);
            assert_eq!(
                Args::try_parse_from(&["em", "--list", "input-formats"])
                    .unwrap()
                    .list_info,
                Some(RequestedInfo::InputFormats)
            );
            assert_eq!(
                Args::try_parse_from(&["em", "--list", "input-extensions"])
                    .unwrap()
                    .list_info,
                Some(RequestedInfo::InputExtensions)
            );
            assert_eq!(
                Args::try_parse_from(&["em", "--list", "output-formats"])
                    .unwrap()
                    .list_info,
                Some(RequestedInfo::OutputFormats)
            );
            assert_eq!(
                Args::try_parse_from(&["em", "--list", "output-extensions"])
                    .unwrap()
                    .list_info,
                Some(RequestedInfo::OutputExtensions)
            );
            assert!(Args::try_parse_from(&["em", "--list", "root-passwd"]).is_err());
        }

        #[test]
        fn max_mem() {
            assert_eq!(
                Args::try_parse_from(&["em"]).unwrap().max_mem,
                MemoryLimit::Unlimited
            );
            assert_eq!(
                Args::try_parse_from(&["em", "--max-mem", "25"])
                    .unwrap()
                    .max_mem,
                MemoryLimit::Limited(25)
            );
            assert_eq!(
                Args::try_parse_from(&["em", "--max-mem", "25K"])
                    .unwrap()
                    .max_mem,
                MemoryLimit::Limited(25 * 1024)
            );
            assert_eq!(
                Args::try_parse_from(&["em", "--max-mem", "25M"])
                    .unwrap()
                    .max_mem,
                MemoryLimit::Limited(25 * 1024 * 1024)
            );
            assert_eq!(
                Args::try_parse_from(&["em", "--max-mem", "25G"])
                    .unwrap()
                    .max_mem,
                MemoryLimit::Limited(25 * 1024 * 1024 * 1024)
            );

            assert!(Args::try_parse_from(&["em", "--max-mem", "100T"]).is_err());
        }

        #[test]
        fn style() {
            assert_eq!(Args::try_parse_from(&["em"]).unwrap().style, None);
            assert_eq!(
                Args::try_parse_from(&["em", "-s", "funk"]).unwrap().style,
                Some("funk".to_owned())
            );
        }

        #[test]
        fn sandbox() {
            assert_eq!(
                Args::try_parse_from(&["em"]).unwrap().sandbox,
                SandboxLevel::Standard
            );
            assert_eq!(
                Args::try_parse_from(&["em", "--sandbox", "unrestricted"])
                    .unwrap()
                    .sandbox,
                SandboxLevel::Unrestricted
            );
            assert_eq!(
                Args::try_parse_from(&["em", "--sandbox", "standard"])
                    .unwrap()
                    .sandbox,
                SandboxLevel::Standard
            );
            assert_eq!(
                Args::try_parse_from(&["em", "--sandbox", "strict"])
                    .unwrap()
                    .sandbox,
                SandboxLevel::Strict
            );

            assert!(Args::try_parse_from(&["em", "--sandbox", "root"]).is_err());
        }

        #[test]
        fn style_path() {
            assert_eq!(
                Args::try_parse_from(&["em"]).unwrap().style_path,
                SearchPath::default()
            );
            assert_eq!(
                Args::try_parse_from(&["em", "--style-path", "club:house"])
                    .unwrap()
                    .style_path,
                SearchPath::from(vec!["club".to_owned(), "house".to_owned()])
            );
        }

        #[test]
        fn verbosity() {
            assert_eq!(
                {
                    let empty: [&str; 0] = [];
                    Args::try_parse_from(empty).unwrap().verbosity
                },
                Verbosity::Terse
            );
            assert_eq!(
                Args::try_parse_from(["em"]).unwrap().verbosity,
                Verbosity::Terse
            );
            assert_eq!(
                Args::try_parse_from(["em", "-v"]).unwrap().verbosity,
                Verbosity::Verbose
            );
            assert_eq!(
                Args::try_parse_from(["em", "-vv"]).unwrap().verbosity,
                Verbosity::Debug
            );
            assert!(Args::try_parse_from(["em", "-vvv"]).is_err());
        }

        #[test]
        fn extensions() {
            let empty: [&str; 0] = [];
            assert_eq!(Args::try_parse_from(["em"]).unwrap().extensions, empty);
            assert_eq!(
                Args::try_parse_from(["em", "-x", "foo", "-x", "bar", "-x", "baz"])
                    .unwrap()
                    .extensions,
                ["foo".to_owned(), "bar".to_owned(), "baz".to_owned()]
            );
        }

        #[test]
        fn extension_args() {
            assert_eq!(
                Args::try_parse_from(&["em"]).unwrap().extension_args,
                vec![]
            );

            {
                let valid_ext_args = Args::try_parse_from(&["em", "-ak=v", "-ak2=v2", "-ak3="])
                    .unwrap()
                    .extension_args;
                assert_eq!(valid_ext_args.len(), 3);
                assert_eq!(valid_ext_args[0].name(), "k");
                assert_eq!(valid_ext_args[0].value(), "v");
                assert_eq!(valid_ext_args[1].name(), "k2");
                assert_eq!(valid_ext_args[1].value(), "v2");
                assert_eq!(valid_ext_args[2].name(), "k3");
                assert_eq!(valid_ext_args[2].value(), "");
            }

            assert!(Args::try_parse_from(&["em", "-a=v"]).is_err());
        }

        #[test]
        fn extension_path() {
            assert_eq!(
                Args::try_parse_from(&["em"]).unwrap().extension_path,
                SearchPath::default()
            );
            assert_eq!(
                Args::try_parse_from(&["em", "--extension-path", "club:house"])
                    .unwrap()
                    .extension_path,
                SearchPath::from(vec!["club".to_owned(), "house".to_owned()])
            );
        }
    }

    mod source_path {
        use super::*;

        #[test]
        fn try_from() {
            assert_eq!(
                UninferredArgPath::try_from("foo").unwrap(),
                UninferredArgPath::Path(path::PathBuf::from("foo"))
            );
        }

        #[test]
        fn infer_input() {
            assert_eq!(
                UninferredArgPath::Infer.infer_input(),
                ArgPath::from("main")
            );
            assert_eq!(UninferredArgPath::Stdio.infer_input(), ArgPath::Stdio);
            assert_eq!(
                UninferredArgPath::try_from("62 West Wallaby St.")
                    .ok()
                    .unwrap()
                    .infer_input(),
                ArgPath::Path(path::PathBuf::from("62 West Wallaby St."))
            );
        }

        #[test]
        fn infer_output() {
            let resolved_path = ArgPath::from("main");
            let resolved_stdio = ArgPath::Stdio;

            assert_eq!(
                UninferredArgPath::Infer.infer_output(&resolved_path),
                ArgPath::from("main")
            );
            assert_eq!(
                UninferredArgPath::Infer.infer_output(&resolved_stdio),
                ArgPath::Stdio
            );
            assert_eq!(
                UninferredArgPath::Stdio.infer_output(&resolved_path),
                ArgPath::Stdio
            );
            assert_eq!(
                UninferredArgPath::Stdio.infer_output(&resolved_stdio),
                ArgPath::Stdio
            );
            assert_eq!(
                UninferredArgPath::try_from("Tottington Hall")
                    .ok()
                    .unwrap()
                    .infer_output(&resolved_path),
                ArgPath::from("Tottington Hall")
            );
            assert_eq!(
                UninferredArgPath::try_from("Tottington Hall")
                    .ok()
                    .unwrap()
                    .infer_output(&resolved_stdio),
                ArgPath::from("Tottington Hall")
            );
        }
    }

    mod search_path {
        use super::*;
        use std::io::Read;
        #[test]
        fn search_path_from() {
            assert_eq!(
                SearchPath::from("foo:bar::baz"),
                SearchPath {
                    path: vec!["foo", "bar", "baz"].iter().map(|d| d.into()).collect()
                }
            );

            assert_eq!(
                SearchPath::from("foo:bar::baz".to_owned()),
                SearchPath {
                    path: vec!["foo", "bar", "baz"].iter().map(|d| d.into()).collect()
                }
            );

            assert_eq!(
                SearchPath::from(
                    vec!["foo", "bar", "baz"]
                        .iter()
                        .map(|d| path::PathBuf::from(d))
                        .collect::<Vec<_>>()
                ),
                SearchPath {
                    path: vec!["foo", "bar", "baz"].iter().map(|d| d.into()).collect()
                }
            );
        }

        #[test]
        fn to_string() {
            let path = SearchPath::from("asdf:fdsa: ::q");
            assert_eq!(path.to_string(), "asdf:fdsa: :q");
        }

        fn make_file(tmppath: &path::Path, filepath: &str, content: &str) -> Result<(), io::Error> {
            let path = path::PathBuf::from(tmppath).join(filepath);

            if let Some(parent) = path.parent() {
                fs::create_dir_all(parent)?;
            }

            fs::write(path, content)
        }

        #[test]
        fn open() -> Result<(), io::Error> {
            let tmpdir = tempfile::tempdir()?;
            let tmppath = tmpdir.path().canonicalize()?;

            make_file(&tmppath, "a.txt", "a")?;
            make_file(&tmppath, "B/b.txt", "b")?;
            make_file(&tmppath, "C1/C2/c.txt", "c")?;
            make_file(&tmppath, "D/d.txt", "c")?;
            make_file(&tmppath, "x.txt", "x")?;

            let raw_path: Vec<path::PathBuf> = vec!["B", "C1", "D"]
                .iter()
                .map(|s| path::PathBuf::from(&tmppath).join(s))
                .collect();
            let path = SearchPath::from(raw_path).normalised();

            {
                let a = path.open(&tmppath, "a.txt");
                assert!(a.is_ok(), "{:?}", a);
                let mut content = String::new();
                let found = a.unwrap();
                assert_eq!(found.path(), tmppath.join("a.txt"));
                found.file().read_to_string(&mut content)?;
                assert_eq!(content, "a");
            }

            {
                let b = path.open(&tmppath, "b.txt");
                assert!(b.is_ok(), "{:?}", b);
                let found = b.unwrap();
                assert_eq!(found.path(), tmppath.join("B/b.txt"));
                let mut content = String::new();
                found.file().read_to_string(&mut content)?;
                assert_eq!(content, "b");
            }

            {
                let c = path.open(&tmppath, "C2/c.txt");
                assert!(c.is_ok());
                let found = c.unwrap();
                assert_eq!(found.path(), tmppath.join("C1/C2/c.txt"));
                let mut content = String::new();
                found.file().read_to_string(&mut content)?;
                assert_eq!(content, "c");
            }

            {
                let c = path.open(&tmppath, "D/d.txt");
                assert!(c.is_ok());
                let found = c.unwrap();
                assert_eq!(found.path(), tmppath.join("D/d.txt"));
                let mut content = String::new();
                found.file().read_to_string(&mut content)?;
                assert_eq!(content, "c");
            }

            {
                let abs_path = tmppath.join("a.txt");
                let abs_result =
                    path.open(&tmppath, &path::PathBuf::from(&abs_path).canonicalize()?);
                assert!(abs_result.is_err());
                let err = abs_result.unwrap_err();
                assert_eq!(err.kind(), io::ErrorKind::InvalidInput);
                assert_eq!(
                    err.to_string(),
                    format!("Absolute paths are forbidden: got {:?}", abs_path,)
                );
            }

            {
                let dir_result = path.open(&tmppath, "D");
                assert!(dir_result.is_err());
                let err = dir_result.unwrap_err();
                assert_eq!(err.kind(), io::ErrorKind::NotFound);
                assert_eq!(
                    err.to_string(),
                    format!(
                        "Could not find file \"D\" along path \"{}\"",
                        path.to_string()
                    )
                );
            }

            {
                let dir_result = path.open(&tmppath, "C2");
                assert!(dir_result.is_err());
                let err = dir_result.unwrap_err();
                assert_eq!(err.kind(), io::ErrorKind::NotFound);
                assert_eq!(
                    err.to_string(),
                    format!(
                        "Could not find file \"C2\" along path \"{}\"",
                        path.to_string()
                    )
                );
            }

            {
                let inaccessible = path.open(&tmppath, "c.txt");
                assert!(inaccessible.is_err());
                let err = inaccessible.unwrap_err();
                assert_eq!(err.kind(), io::ErrorKind::NotFound);
                assert_eq!(
                    err.to_string(),
                    format!(
                        "Could not find file \"c.txt\" along path \"{}\"",
                        path.to_string()
                    )
                );
            }

            {
                let inaccessible = path.open(&tmppath, "../a.txt");
                assert!(inaccessible.is_err());
                let abs_file = inaccessible.unwrap_err();
                assert_eq!(abs_file.kind(), io::ErrorKind::NotFound);
                assert_eq!(
                    abs_file.to_string(),
                    format!(
                        "Could not find file \"../a.txt\" along path \"{}\"",
                        path.to_string()
                    )
                );
            }

            {
                let non_existent = path.open(&tmppath, "non-existent.txt");
                assert!(non_existent.is_err());
                let non_existent = non_existent.unwrap_err();
                assert_eq!(non_existent.kind(), io::ErrorKind::NotFound);
                assert_eq!(
                    non_existent.to_string(),
                    format!(
                        "Could not find file \"non-existent.txt\" along path \"{}\"",
                        path.to_string()
                    )
                );
            }

            Ok(())
        }
    }
    mod search_result {
        use super::*;
        use io::Write;

        #[test]
        fn fields() -> io::Result<()> {
            let tmpdir = tempfile::tempdir()?;
            let path = tmpdir.path().join("fields.txt");
            let mut file = fs::File::create(&path)?;
            file.write(b"asdf")?;

            let s = SearchResult::new(path.clone(), file.try_clone()?);

            assert_eq!(s.path(), &path);
            assert_eq!(
                s.file().metadata().unwrap().len(),
                file.metadata().unwrap().len()
            );

            Ok(())
        }
    }
}
