use clap::{
    builder::{OsStr, StringValueParser, TypedValueParser},
    error,
    ArgAction::{Append, Count, Help, Version},
    CommandFactory, Parser, Subcommand, ValueEnum,
    ValueHint::{AnyPath, DirPath, FilePath},
};
#[cfg(test)]
use std::io::BufReader;
use std::{
    env,
    ffi::OsString,
    fmt::Display,
    fs::{self, File},
    io::{self, Read, Stdin},
    os::fd::AsRawFd,
    path,
};

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

#[derive(Debug)]
pub struct LogArgs {
    /// Colourise log messages
    pub colour: bool,

    /// Make warnings into errors
    pub warnings_as_errors: bool,

    /// Output verbosity
    pub verbosity: Verbosity,
}

impl TryFrom<RawLogArgs> for LogArgs {
    type Error = clap::Error;

    fn try_from(raw: RawLogArgs) -> Result<Self, Self::Error> {
        let RawLogArgs {
            colour,
            warnings_as_errors,
            verbosity,
        } = raw;
        Ok(Self {
            colour: colour.into(),
            warnings_as_errors,
            verbosity: verbosity.try_into()?,
        })
    }
}

const LONG_ABOUT: &str = "Takes input of a markdown-like document, processes it and typesets it before passing the result to a driver for outputting in some format. Extensions can be used to include arbitrary functionality; device drivers are also extensions.";

/// Internal command-line argument parser
#[derive(Parser, Debug)]
#[command(author, version, about, long_about=LONG_ABOUT, disable_help_flag=true, disable_version_flag=true)]
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

#[derive(Debug, Parser)]
pub struct RawLogArgs {
    /// Colourise log messages
    #[arg(long, value_enum, default_value_t, value_name = "when", global = true)]
    colour: ColouriseOutput,

    /// Make warnings into errors
    #[arg(short = 'E', default_value_t = false, global = true)]
    warnings_as_errors: bool,

    /// Set output verbosity
    #[arg(short, action=Count, default_value_t=0, value_name = "level", global=true)]
    verbosity: u8,
}

/// What emblem will do this execution
#[derive(Clone, Debug, PartialEq, Eq, Subcommand)]
#[warn(missing_docs)]
pub enum Command {
    /// Build a given document
    Build(BuildCmd),

    /// Explain a given error
    Explain(ExplainCmd),

    /// Fix formatting errors in the given document
    #[command(name = "fmt")]
    Format(FormatCmd),

    /// Create a new emblem document
    Init(InitCmd),

    /// Check for linting errors in the given document
    Lint(LintCmd),

    /// Print info and exit
    List(ListCmd),
}

#[cfg(test)]
impl Command {
    fn build(&self) -> Option<&BuildCmd> {
        match self {
            Self::Build(b) => Some(b),
            _ => None,
        }
    }

    fn explain(&self) -> Option<&ExplainCmd> {
        match self {
            Self::Explain(e) => Some(e),
            _ => None,
        }
    }

    fn format(&self) -> Option<&FormatCmd> {
        match self {
            Self::Format(f) => Some(f),
            _ => None,
        }
    }

    fn init(&self) -> Option<&InitCmd> {
        match self {
            Self::Init(i) => Some(i),
            _ => None,
        }
    }

    fn lint(&self) -> Option<&LintCmd> {
        match self {
            Self::Lint(l) => Some(l),
            _ => None,
        }
    }

    fn list(&self) -> Option<&ListCmd> {
        match self {
            Self::List(l) => Some(l),
            _ => None,
        }
    }
}

impl Default for Command {
    fn default() -> Self {
        Self::Build(BuildCmd::default())
    }
}

/// Arguments to the build subcommand
#[derive(Clone, Debug, Default, Parser, PartialEq, Eq)]
#[warn(missing_docs)]
pub struct BuildCmd {
    #[command(flatten)]
    #[allow(missing_docs)]
    pub input: InputArgs,

    #[command(flatten)]
    #[allow(missing_docs)]
    pub output: OutputArgs,

    #[command(flatten)]
    #[allow(missing_docs)]
    pub extensions: ExtensionArgs,

    #[command(flatten)]
    #[allow(missing_docs)]
    pub style: StyleArgs,
}

impl BuildCmd {
    #[allow(dead_code)]
    pub fn output_stem(&self) -> ArgPath {
        self.output.stem.infer_from(&self.input.file)
    }
}

/// Arguments to the explain subcommand
#[derive(Clone, Debug, Parser, PartialEq, Eq)]
#[warn(missing_docs)]
pub struct ExplainCmd {
    /// Code of the error to explain
    #[arg(value_name = "error-code")]
    pub id: String,
}

/// Arguments to the fmt subcommand
#[derive(Clone, Debug, Parser, PartialEq, Eq)]
#[warn(missing_docs)]
pub struct FormatCmd {
    #[command(flatten)]
    #[allow(missing_docs)]
    pub input: InputArgs,
}

/// Arguments to the init subcommand
#[derive(Clone, Debug, Parser, PartialEq, Eq)]
#[warn(missing_docs)]
pub struct InitCmd {
    /// Directory to contain the new document
    #[arg(value_name = "dir", value_hint = DirPath, default_value = ".")]
    dir: String,

    /// Allow writing to non-empty directories
    #[arg(long)]
    dir_not_empty: bool,
}

impl InitCmd {
    pub fn dir(&self) -> &str {
        &self.dir
    }

    pub fn dir_not_empty(&self) -> bool {
        self.dir_not_empty
    }
}

#[cfg(test)]
impl InitCmd {
    pub fn new(dir: String, dir_not_empty: bool) -> Self {
        Self { dir, dir_not_empty }
    }
}

/// Arguments to the lint subcommand
#[derive(Clone, Debug, Parser, PartialEq, Eq)]
#[warn(missing_docs)]
pub struct LintCmd {
    #[command(flatten)]
    #[allow(missing_docs)]
    pub input: InputArgs,

    #[command(flatten)]
    #[allow(missing_docs)]
    pub extensions: ExtensionArgs,
}

/// Arguments to the list subcommand
#[derive(Clone, Debug, Parser, PartialEq, Eq)]
#[warn(missing_docs)]
pub struct ListCmd {
    /// What to list
    #[arg(value_enum, value_name = "what")]
    pub what: RequestedInfo,

    #[command(flatten)]
    #[allow(missing_docs)]
    pub extensions: ExtensionArgs,
}

/// Holds the source of the user's document
#[derive(Clone, Debug, Default, Parser, PartialEq, Eq)]
#[warn(missing_docs)]
pub struct InputArgs {
    /// Document to typeset
    #[arg(value_name = "in-file", value_hint = FilePath, default_value_t = ArgPath::default(), value_parser = ArgPath::parser())]
    pub file: ArgPath,
}

/// Holds where and how the user wants their output
#[derive(Clone, Debug, Default, Parser, PartialEq, Eq)]
#[warn(missing_docs)]
pub struct OutputArgs {
    /// Output file path
    #[arg(value_name = "out-file", value_hint = AnyPath, default_value_t=UninferredArgPath::default(), value_parser = UninferredArgPath::parser())]
    stem: UninferredArgPath,

    /// Override detected output format
    #[arg(short = 'T', value_name = "format")]
    pub driver: Option<String>,
}

/// Holds the user's preferences for the extensions used when running the program
#[derive(Clone, Debug, Default, Parser, PartialEq, Eq)]
#[warn(missing_docs)]
pub struct ExtensionArgs {
    /// Pass variable into extension-space
    #[arg(short = 'a', action = Append, value_parser = ExtArg::parser(),  value_name="arg=value")]
    pub args: Vec<ExtArg>,

    /// Extension search-path, colon-separated
    #[arg(id="extension-path", long, env = "EM_EXT_PATH", value_parser = SearchPath::parser(), default_value = "", value_name = "path")]
    pub path: SearchPath,

    /// Limit lua memory usage
    #[arg(long, value_parser = MemoryLimit::parser(), default_value = "unlimited", value_name = "amount")]
    pub max_mem: MemoryLimit,

    /// Restrict system access
    #[arg(long, value_enum, default_value_t, value_name = "level")]
    pub sandbox: SandboxLevel,

    /// Load an extension
    #[arg(short = 'x', action=Append, value_name = "ext")]
    pub list: Vec<String>,
}

/// Holds the user's preferences for the style of their document
#[derive(Clone, Debug, Default, Parser, PartialEq, Eq)]
#[warn(missing_docs)]
pub struct StyleArgs {
    /// Set root stylesheet
    #[arg(short = 's', value_name = "style")]
    pub name: Option<String>,

    /// Style search-path, colon-separated
    #[arg(id="style-path", long, env = "EM_STYLE_PATH", value_parser = SearchPath::parser(), default_value = "", value_name = "path")]
    pub path: SearchPath,
}

#[derive(Clone, Default, Debug, PartialEq, Eq)]
pub enum UninferredArgPath {
    #[default]
    Infer,
    Stdio,
    Path(path::PathBuf),
}

impl UninferredArgPath {
    fn parser() -> impl TypedValueParser {
        StringValueParser::new().try_map(Self::try_from)
    }

    fn infer_from(&self, other: &ArgPath) -> ArgPath {
        match self {
            Self::Infer => match other {
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

const FILE_PATH_CANNOT_BE_EMPTY: &str = "file path cannot be empty";

impl TryFrom<&str> for UninferredArgPath {
    type Error = error::Error;

    fn try_from(raw: &str) -> Result<Self, Self::Error> {
        match raw {
            "" => {
                Err(RawArgs::command()
                    .error(error::ErrorKind::InvalidValue, FILE_PATH_CANNOT_BE_EMPTY))
            }
            "-" => Ok(Self::Stdio),
            "??" => Ok(Self::Infer),
            path => Ok(Self::Path(path.into())),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ArgPath {
    Stdio,
    Path(path::PathBuf),
}

impl ArgPath {
    fn parser() -> impl TypedValueParser {
        StringValueParser::new().try_map(Self::try_from)
    }

    #[allow(dead_code)]
    pub fn path(&self) -> Option<&path::Path> {
        match self {
            Self::Path(p) => Some(p),
            Self::Stdio => None,
        }
    }
}

impl Default for ArgPath {
    fn default() -> Self {
        Self::Path("main.em".into())
    }
}

impl TryFrom<OsStr> for ArgPath {
    type Error = error::Error;

    fn try_from(raw: OsStr) -> Result<Self, Self::Error> {
        if let Some(s) = raw.to_str() {
            return Self::try_from(s);
        }
        Err(RawArgs::command().error(
            error::ErrorKind::InvalidValue,
            format!("could not convert '{:?}' to a valid UTF-8 string", raw),
        ))
    }
}

impl TryFrom<String> for ArgPath {
    type Error = error::Error;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        Self::try_from(&value[..])
    }
}

impl TryFrom<&str> for ArgPath {
    type Error = clap::Error;

    fn try_from(raw: &str) -> Result<Self, Self::Error> {
        match raw {
            "" => {
                Err(RawArgs::command()
                    .error(error::ErrorKind::InvalidValue, FILE_PATH_CANNOT_BE_EMPTY))
            }
            "-" => Ok(Self::Stdio),
            raw => Ok(Self::Path(path::PathBuf::from(raw))),
        }
    }
}

impl Display for ArgPath {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Self::Stdio => "-",
                Self::Path(s) => s.to_str().unwrap_or("(invalid path)"),
            }
        )
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

    #[allow(dead_code)]
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

        let path = path::PathBuf::from(&src).join(target);
        if path.starts_with(&src) {
            if let Ok(file) = fs::File::open(&path) {
                if let Ok(metadata) = file.metadata() {
                    if metadata.is_file() {
                        let file = InputFile::from(file);
                        return Ok(SearchResult { path, file });
                    }
                }
            }
        }

        for dir in self.normalised().path {
            let path = {
                let p = path::PathBuf::from(&dir).join(target);
                match p.canonicalize() {
                    Ok(p) => p,
                    _ => continue,
                }
            };

            if !path.starts_with(&dir) {
                continue;
            }

            if let Ok(file) = fs::File::open(&path) {
                if let Ok(metadata) = file.metadata() {
                    if metadata.is_file() {
                        let file = InputFile::from(file);
                        return Ok(SearchResult { path, file });
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

#[derive(Debug)]
pub struct SearchResult {
    pub(crate) path: path::PathBuf,
    file: InputFile,
}

impl SearchResult {
    pub fn file(&mut self) -> &mut InputFile {
        &mut self.file
    }
}

impl TryFrom<&str> for SearchResult {
    type Error = io::Error;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        Ok(Self {
            path: path::PathBuf::from(value),
            file: InputFile::from(fs::File::open(value)?),
        })
    }
}

impl TryFrom<&ArgPath> for SearchResult {
    type Error = io::Error;

    fn try_from(value: &ArgPath) -> Result<Self, Self::Error> {
        Ok(match value {
            ArgPath::Path(p) => Self {
                path: path::PathBuf::from(p),
                file: InputFile::from(fs::File::open(p)?),
            },
            ArgPath::Stdio => Self {
                path: path::PathBuf::from("-"),
                file: InputFile::from(io::stdin()), // TODO(kcza): lock this!
            },
        })
    }
}

#[derive(Debug)]
pub enum InputFile {
    Stdin(Stdin),
    File(File),
}

impl InputFile {
    pub fn len_hint(&self) -> Option<u64> {
        match self {
            Self::File(f) => f.metadata().ok().map(|m| m.len()),
            Self::Stdin(_) => None,
        }
    }
}

impl PartialEq for InputFile {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::Stdin(_), Self::Stdin(_)) => false,
            (Self::File(f1), Self::File(f2)) => f1.as_raw_fd() == f2.as_raw_fd(),
            _ => false,
        }
    }
}

impl From<File> for InputFile {
    fn from(f: File) -> Self {
        Self::File(f)
    }
}

impl From<Stdin> for InputFile {
    fn from(stdin: Stdin) -> Self {
        Self::Stdin(stdin)
    }
}

impl Read for InputFile {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        match self {
            Self::Stdin(s) => s.read(buf),
            Self::File(f) => f.read(buf),
        }
    }
}

#[cfg(test)]
impl InputFile {
    fn stdin(&self) -> Option<&Stdin> {
        match self {
            Self::Stdin(s) => Some(s),
            _ => None,
        }
    }

    fn file(&self) -> Option<BufReader<&File>> {
        match self {
            Self::File(f) => Some(BufReader::new(f)),
            _ => None,
        }
    }
}

#[derive(ValueEnum, Clone, Debug, Eq, PartialEq)]
pub enum RequestedInfo {
    // InputFormats,
    // InputExtensions,
    OutputFormats,
    OutputExtensions,
}

#[derive(ValueEnum, Copy, Clone, Debug, Default, Eq, PartialEq, Ord, PartialOrd)]
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

#[derive(ValueEnum, Copy, Clone, Debug, Default, PartialEq, Eq)]
pub enum ColouriseOutput {
    Never,
    #[default]
    Auto,
    Always,
}

impl From<ColouriseOutput> for bool {
    fn from(hint: ColouriseOutput) -> Self {
        use supports_color::Stream;

        match hint {
            ColouriseOutput::Always => true,
            ColouriseOutput::Auto => {
                if let Some(support) = supports_color::on(Stream::Stderr) {
                    support.has_basic
                } else {
                    false
                }
            }
            ColouriseOutput::Never => false,
        }
    }
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

    #[allow(dead_code)]
    pub fn name(&self) -> &str {
        &self.raw[..self.eq_idx]
    }

    #[allow(dead_code)]
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
        fn default() {
            assert_eq!(
                Args::try_parse_from(&["em"]).unwrap().command,
                Args::try_parse_from(&["em", "build"]).unwrap().command
            );
        }

        mod common {
            use super::*;

            #[test]
            fn colourise_output() {
                assert_eq!(
                    Args::try_parse_from(&["em", "--colour", "never"])
                        .unwrap()
                        .log
                        .colour,
                    false
                );
                assert!(
                    Args::try_parse_from(&["em", "--colour", "always"])
                        .unwrap()
                        .log
                        .colour
                );

                assert!(Args::try_parse_from(&["em", "--colour", "crabcakes"]).is_err());
            }

            #[test]
            fn warnings_as_errors() {
                assert!(
                    !Args::try_parse_from(&["em"])
                        .unwrap()
                        .log
                        .warnings_as_errors
                );
                assert!(
                    Args::try_parse_from(&["em", "-E"])
                        .unwrap()
                        .log
                        .warnings_as_errors
                );
            }

            #[test]
            fn verbosity() {
                assert_eq!(
                    {
                        let empty: [&str; 0] = [];
                        Args::try_parse_from(empty).unwrap().log.verbosity
                    },
                    Verbosity::Terse
                );
                assert_eq!(
                    Args::try_parse_from(["em"]).unwrap().log.verbosity,
                    Verbosity::Terse
                );
                assert_eq!(
                    Args::try_parse_from(["em", "-v"]).unwrap().log.verbosity,
                    Verbosity::Verbose
                );
                assert_eq!(
                    Args::try_parse_from(["em", "-vv"]).unwrap().log.verbosity,
                    Verbosity::Debug
                );
                assert!(Args::try_parse_from(["em", "-vvv"]).is_err());
            }
        }

        mod build {
            use super::*;

            #[test]
            fn output_driver() {
                assert_eq!(
                    Args::try_parse_from(&["em"])
                        .unwrap()
                        .command
                        .build()
                        .unwrap()
                        .output
                        .driver,
                    None
                );
                assert_eq!(
                    Args::try_parse_from(&["em", "build", "-T", "pies"])
                        .unwrap()
                        .command
                        .build()
                        .unwrap()
                        .output
                        .driver,
                    Some("pies".to_owned())
                );
            }

            #[test]
            fn input_file() {
                assert_eq!(
                    Args::try_parse_from(&["em"])
                        .unwrap()
                        .command
                        .build()
                        .unwrap()
                        .input
                        .file,
                    ArgPath::try_from("main.em").unwrap(),
                );
                assert_eq!(
                    Args::try_parse_from(&["em", "build", "-"])
                        .unwrap()
                        .command
                        .build()
                        .unwrap()
                        .input
                        .file,
                    ArgPath::Stdio
                );
                assert_eq!(
                    Args::try_parse_from(&["em", "build", "chickens"])
                        .unwrap()
                        .command
                        .build()
                        .unwrap()
                        .input
                        .file,
                    ArgPath::try_from("chickens").unwrap(),
                );
            }

            #[test]
            fn output_stem() {
                assert_eq!(
                    Args::try_parse_from(&["em"])
                        .unwrap()
                        .command
                        .build()
                        .unwrap()
                        .output_stem(),
                    ArgPath::try_from("main.em").unwrap(),
                );
                assert_eq!(
                    Args::try_parse_from(&["em", "build", "-"])
                        .unwrap()
                        .command
                        .build()
                        .unwrap()
                        .output_stem(),
                    ArgPath::Stdio,
                );
                assert_eq!(
                    Args::try_parse_from(&["em", "build", "-", "pies"])
                        .unwrap()
                        .command
                        .build()
                        .unwrap()
                        .output_stem(),
                    ArgPath::try_from("pies").unwrap(),
                );
                assert_eq!(
                    Args::try_parse_from(&["em", "build", "_", "-"])
                        .unwrap()
                        .command
                        .build()
                        .unwrap()
                        .output_stem(),
                    ArgPath::Stdio,
                );
                assert_eq!(
                    Args::try_parse_from(&["em", "build", "_", "pies"])
                        .unwrap()
                        .command
                        .build()
                        .unwrap()
                        .output_stem(),
                    ArgPath::try_from("pies").unwrap(),
                );
                assert_eq!(
                    Args::try_parse_from(&["em", "build", "-", "pies"])
                        .unwrap()
                        .command
                        .build()
                        .unwrap()
                        .output_stem(),
                    ArgPath::try_from("pies").unwrap(),
                );
            }

            #[test]
            fn max_mem() {
                assert_eq!(
                    Args::try_parse_from(&["em"])
                        .unwrap()
                        .command
                        .build()
                        .unwrap()
                        .extensions
                        .max_mem,
                    MemoryLimit::Unlimited
                );
                assert_eq!(
                    Args::try_parse_from(&["em", "build", "--max-mem", "25"])
                        .unwrap()
                        .command
                        .build()
                        .unwrap()
                        .extensions
                        .max_mem,
                    MemoryLimit::Limited(25)
                );
                assert_eq!(
                    Args::try_parse_from(&["em", "build", "--max-mem", "25K"])
                        .unwrap()
                        .command
                        .build()
                        .unwrap()
                        .extensions
                        .max_mem,
                    MemoryLimit::Limited(25 * 1024)
                );
                assert_eq!(
                    Args::try_parse_from(&["em", "build", "--max-mem", "25M"])
                        .unwrap()
                        .command
                        .build()
                        .unwrap()
                        .extensions
                        .max_mem,
                    MemoryLimit::Limited(25 * 1024 * 1024)
                );
                assert_eq!(
                    Args::try_parse_from(&["em", "build", "--max-mem", "25G"])
                        .unwrap()
                        .command
                        .build()
                        .unwrap()
                        .extensions
                        .max_mem,
                    MemoryLimit::Limited(25 * 1024 * 1024 * 1024)
                );

                assert!(Args::try_parse_from(&["em", "build", "--max-mem", "100T"]).is_err());
            }

            #[test]
            fn style() {
                assert_eq!(
                    Args::try_parse_from(&["em"])
                        .unwrap()
                        .command
                        .build()
                        .unwrap()
                        .style
                        .name,
                    None
                );
                assert_eq!(
                    Args::try_parse_from(&["em", "build", "-s", "funk"])
                        .unwrap()
                        .command
                        .build()
                        .unwrap()
                        .style
                        .name,
                    Some("funk".to_owned())
                );
            }

            #[test]
            fn sandbox() {
                assert_eq!(
                    Args::try_parse_from(&["em"])
                        .unwrap()
                        .command
                        .build()
                        .unwrap()
                        .extensions
                        .sandbox,
                    SandboxLevel::Standard
                );
                assert_eq!(
                    Args::try_parse_from(&["em", "build", "--sandbox", "unrestricted"])
                        .unwrap()
                        .command
                        .build()
                        .unwrap()
                        .extensions
                        .sandbox,
                    SandboxLevel::Unrestricted
                );
                assert_eq!(
                    Args::try_parse_from(&["em", "build", "--sandbox", "standard"])
                        .unwrap()
                        .command
                        .build()
                        .unwrap()
                        .extensions
                        .sandbox,
                    SandboxLevel::Standard
                );
                assert_eq!(
                    Args::try_parse_from(&["em", "build", "--sandbox", "strict"])
                        .unwrap()
                        .command
                        .build()
                        .unwrap()
                        .extensions
                        .sandbox,
                    SandboxLevel::Strict
                );

                assert!(Args::try_parse_from(&["em", "build", "--sandbox", "root"]).is_err());
            }

            #[test]
            fn style_path() {
                assert_eq!(
                    Args::try_parse_from(&["em"])
                        .unwrap()
                        .command
                        .build()
                        .unwrap()
                        .style
                        .path,
                    SearchPath::default()
                );
                assert_eq!(
                    Args::try_parse_from(&["em", "build", "--style-path", "club:house"])
                        .unwrap()
                        .command
                        .build()
                        .unwrap()
                        .style
                        .path,
                    SearchPath::from(vec!["club".to_owned(), "house".to_owned()])
                );
            }

            #[test]
            fn extensions() {
                let empty: [&str; 0] = [];
                assert_eq!(
                    Args::try_parse_from(["em"])
                        .unwrap()
                        .command
                        .build()
                        .unwrap()
                        .extensions
                        .list,
                    empty
                );
                assert_eq!(
                    Args::try_parse_from(["em", "build", "-x", "foo", "-x", "bar", "-x", "baz"])
                        .unwrap()
                        .command
                        .build()
                        .unwrap()
                        .extensions
                        .list,
                    ["foo".to_owned(), "bar".to_owned(), "baz".to_owned()]
                );
            }

            #[test]
            fn extension_args() {
                assert_eq!(
                    Args::try_parse_from(&["em"])
                        .unwrap()
                        .command
                        .build()
                        .unwrap()
                        .extensions
                        .args,
                    vec![]
                );

                {
                    let valid_ext_args =
                        Args::try_parse_from(&["em", "build", "-ak=v", "-ak2=v2", "-ak3="])
                            .unwrap()
                            .command
                            .build()
                            .unwrap()
                            .extensions
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

                assert!(Args::try_parse_from(&["em", "-a=v"]).is_err());
            }

            #[test]
            fn extension_path() {
                assert_eq!(
                    Args::try_parse_from(&["em"])
                        .unwrap()
                        .command
                        .build()
                        .unwrap()
                        .extensions
                        .path,
                    SearchPath::default()
                );
                assert_eq!(
                    Args::try_parse_from(&["em", "build", "--extension-path", "club:house"])
                        .unwrap()
                        .command
                        .build()
                        .unwrap()
                        .extensions
                        .path,
                    SearchPath::from(vec!["club".to_owned(), "house".to_owned()])
                );
            }
        }

        mod explain {
            use super::*;

            #[test]
            fn code() {
                assert_eq!(
                    Args::try_parse_from(&["em", "explain", "E001"])
                        .unwrap()
                        .command
                        .explain()
                        .unwrap()
                        .id,
                    "E001"
                );
                assert!(Args::try_parse_from(&["em", "explain"]).is_err());
            }
        }

        mod format {
            use super::*;

            #[test]
            fn input_file() {
                assert_eq!(
                    Args::try_parse_from(&["em", "fmt"])
                        .unwrap()
                        .command
                        .format()
                        .unwrap()
                        .input
                        .file,
                    ArgPath::Path("main.em".into())
                );
                assert_eq!(
                    Args::try_parse_from(&["em", "fmt", "-"])
                        .unwrap()
                        .command
                        .format()
                        .unwrap()
                        .input
                        .file,
                    ArgPath::Stdio
                );
                assert_eq!(
                    Args::try_parse_from(&["em", "fmt", "plain.txt"])
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

        mod init {
            use super::*;

            #[test]
            fn dir() {
                assert_eq!(
                    Args::try_parse_from(&["em", "init"])
                        .unwrap()
                        .command
                        .init()
                        .unwrap()
                        .dir(),
                    ".",
                );
                assert_eq!(
                    Args::try_parse_from(&["em", "init", "cool-doc"])
                        .unwrap()
                        .command
                        .init()
                        .unwrap()
                        .dir(),
                    "cool-doc",
                );
            }
        }

        mod lint {
            use super::*;

            #[test]
            fn input_file() {
                assert_eq!(
                    Args::try_parse_from(&["em", "lint"])
                        .unwrap()
                        .command
                        .lint()
                        .unwrap()
                        .input
                        .file,
                    ArgPath::Path("main.em".into())
                );
                assert_eq!(
                    Args::try_parse_from(&["em", "lint", "-"])
                        .unwrap()
                        .command
                        .lint()
                        .unwrap()
                        .input
                        .file,
                    ArgPath::Stdio
                );
                assert_eq!(
                    Args::try_parse_from(&["em", "lint", "plain.txt"])
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
            fn extensions() {
                let empty: [&str; 0] = [];
                assert_eq!(
                    Args::try_parse_from(["em", "lint"])
                        .unwrap()
                        .command
                        .lint()
                        .unwrap()
                        .extensions
                        .list,
                    empty
                );
                assert_eq!(
                    Args::try_parse_from(["em", "lint", "-x", "foo", "-x", "bar", "-x", "baz"])
                        .unwrap()
                        .command
                        .lint()
                        .unwrap()
                        .extensions
                        .list,
                    ["foo".to_owned(), "bar".to_owned(), "baz".to_owned()]
                );
            }

            #[test]
            fn extension_args() {
                assert_eq!(
                    Args::try_parse_from(&["em", "lint"])
                        .unwrap()
                        .command
                        .lint()
                        .unwrap()
                        .extensions
                        .args,
                    vec![]
                );

                {
                    let valid_ext_args =
                        Args::try_parse_from(&["em", "lint", "-ak=v", "-ak2=v2", "-ak3="])
                            .unwrap()
                            .command
                            .lint()
                            .unwrap()
                            .extensions
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

                assert!(Args::try_parse_from(&["em", "lint", "-a=v"]).is_err());
            }

            #[test]
            fn extension_path() {
                assert_eq!(
                    Args::try_parse_from(&["em", "lint"])
                        .unwrap()
                        .command
                        .lint()
                        .unwrap()
                        .extensions
                        .path,
                    SearchPath::default()
                );
                assert_eq!(
                    Args::try_parse_from(&["em", "lint", "--extension-path", "club:house"])
                        .unwrap()
                        .command
                        .lint()
                        .unwrap()
                        .extensions
                        .path,
                    SearchPath::from(vec!["club".to_owned(), "house".to_owned()])
                );
            }
        }

        mod list {
            use super::*;

            #[test]
            fn list_info() {
                assert_eq!(
                    Args::try_parse_from(&["em", "list", "output-formats"])
                        .unwrap()
                        .command
                        .list()
                        .unwrap()
                        .what,
                    RequestedInfo::OutputFormats
                );
                assert_eq!(
                    Args::try_parse_from(&["em", "list", "output-extensions"])
                        .unwrap()
                        .command
                        .list()
                        .unwrap()
                        .what,
                    RequestedInfo::OutputExtensions
                );
                assert!(Args::try_parse_from(&["em", "list", "root-passwd"]).is_err());
            }

            #[test]
            fn extensions() {
                let empty: [&str; 0] = [];
                assert_eq!(
                    Args::try_parse_from(["em", "list", "output-formats"])
                        .unwrap()
                        .command
                        .list()
                        .unwrap()
                        .extensions
                        .list,
                    empty
                );
                assert_eq!(
                    Args::try_parse_from([
                        "em",
                        "list",
                        "output-formats",
                        "-x",
                        "foo",
                        "-x",
                        "bar",
                        "-x",
                        "baz"
                    ])
                    .unwrap()
                    .command
                    .list()
                    .unwrap()
                    .extensions
                    .list,
                    ["foo".to_owned(), "bar".to_owned(), "baz".to_owned()]
                );
            }

            #[test]
            fn extension_args() {
                assert_eq!(
                    Args::try_parse_from(&["em", "list", "output-formats"])
                        .unwrap()
                        .command
                        .list()
                        .unwrap()
                        .extensions
                        .args,
                    vec![]
                );

                {
                    let valid_ext_args = Args::try_parse_from(&[
                        "em",
                        "list",
                        "output-formats",
                        "-ak=v",
                        "-ak2=v2",
                        "-ak3=",
                    ])
                    .unwrap()
                    .command
                    .list()
                    .unwrap()
                    .extensions
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

                assert!(Args::try_parse_from(&["em", "list", "-a=v"]).is_err());
            }

            #[test]
            fn extension_path() {
                assert_eq!(
                    Args::try_parse_from(&["em", "list", "output-formats"])
                        .unwrap()
                        .command
                        .list()
                        .unwrap()
                        .extensions
                        .path,
                    SearchPath::default()
                );
                assert_eq!(
                    Args::try_parse_from(&[
                        "em",
                        "list",
                        "output-formats",
                        "--extension-path",
                        "club:house"
                    ])
                    .unwrap()
                    .command
                    .list()
                    .unwrap()
                    .extensions
                    .path,
                    SearchPath::from(vec!["club".to_owned(), "house".to_owned()])
                );
            }
        }
    }

    mod arg_paths {
        use super::*;

        #[test]
        fn try_from() {
            assert_eq!(
                UninferredArgPath::try_from("foo").unwrap(),
                UninferredArgPath::Path(path::PathBuf::from("foo"))
            );
        }

        #[test]
        fn infer_from() {
            let resolved_path = ArgPath::try_from("my-cool-doc.em").unwrap();
            let resolved_stdio = ArgPath::Stdio;

            assert_eq!(
                UninferredArgPath::Infer.infer_from(&resolved_path),
                resolved_path.clone(),
            );
            assert_eq!(
                UninferredArgPath::Infer.infer_from(&resolved_stdio),
                ArgPath::Stdio
            );
            assert_eq!(
                UninferredArgPath::Stdio.infer_from(&resolved_path),
                ArgPath::Stdio
            );
            assert_eq!(
                UninferredArgPath::Stdio.infer_from(&resolved_stdio),
                ArgPath::Stdio
            );
            assert_eq!(
                UninferredArgPath::try_from("Tottington Hall")
                    .ok()
                    .unwrap()
                    .infer_from(&resolved_path),
                ArgPath::try_from("Tottington Hall").unwrap()
            );
            assert_eq!(
                UninferredArgPath::try_from("Tottington Hall")
                    .ok()
                    .unwrap()
                    .infer_from(&resolved_stdio),
                ArgPath::try_from("Tottington Hall").unwrap()
            );
        }

        #[test]
        fn path() {
            assert_eq!(ArgPath::Stdio.path(), None);

            let path = path::PathBuf::from("preston's dog food");
            assert_eq!(ArgPath::Path(path.clone()).path(), Some(path.as_ref()));
        }
    }

    mod search_path {
        use super::*;
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
                let mut found = a.unwrap();
                assert_eq!(found.path, tmppath.join("a.txt"));
                found.file().read_to_string(&mut content)?;
                assert_eq!(content, "a");
            }

            {
                let b = path.open(&tmppath, "b.txt");
                assert!(b.is_ok(), "{:?}", b);
                let mut found = b.unwrap();
                assert_eq!(found.path, tmppath.join("B/b.txt"));
                let mut content = String::new();
                found.file().read_to_string(&mut content)?;
                assert_eq!(content, "b");
            }

            {
                let c = path.open(&tmppath, "C2/c.txt");
                assert!(c.is_ok());
                let mut found = c.unwrap();
                assert_eq!(found.path, tmppath.join("C1/C2/c.txt"));
                let mut content = String::new();
                found.file().read_to_string(&mut content)?;
                assert_eq!(content, "c");
            }

            {
                let c = path.open(&tmppath, "D/d.txt");
                assert!(c.is_ok());
                let mut found = c.unwrap();
                assert_eq!(found.path, tmppath.join("D/d.txt"));
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
            file.write(b"file-content")?;

            let file = fs::File::open(&path)?;
            let mut s = SearchResult {
                path: path.clone(),
                file: InputFile::from(file),
            };

            assert_eq!(s.path, path);
            assert_eq!(
                {
                    let mut buf = String::new();
                    s.file().file().unwrap().read_to_string(&mut buf)?;
                    buf
                },
                "file-content"
            );

            Ok(())
        }

        #[test]
        fn from_str() -> io::Result<()> {
            let src = "from.txt";

            let tmpdir = tempfile::tempdir()?;
            let path = tmpdir.path().join(src);
            let mut file = fs::File::create(&path)?;
            file.write(b"file-content")?;

            let mut s = SearchResult::try_from(path.to_str().unwrap())?;
            assert_eq!(s.path, path);
            assert_eq!(
                {
                    let mut buf = String::new();
                    s.file().file().unwrap().read_to_string(&mut buf)?;
                    buf
                },
                "file-content",
            );

            Ok(())
        }

        #[test]
        fn from_arg_path() -> io::Result<()> {
            let src = "from.txt";

            let tmpdir = tempfile::tempdir()?;
            let path = tmpdir.path().join(src);
            let mut file = fs::File::create(&path)?;
            file.write(b"file-content")?;

            {
                let a = ArgPath::Path(path.clone());
                let mut s = SearchResult::try_from(&a)?;
                assert_eq!(a.path().unwrap(), s.path);
                assert_eq!(
                    {
                        let mut buf = String::new();
                        s.file().file().unwrap().read_to_string(&mut buf)?;
                        buf
                    },
                    "file-content",
                );
            }

            {
                let a = ArgPath::Stdio;
                let mut s = SearchResult::try_from(&a)?;
                assert_eq!(s.path, path::PathBuf::from("-"));
                assert_eq!(
                    s.file().stdin().unwrap().as_raw_fd(),
                    io::stdin().as_raw_fd()
                );
            }

            Ok(())
        }

        #[test]
        fn len_hint() -> io::Result<()> {
            let tmpdir = tempfile::tempdir()?;
            let path = tmpdir.path().join("file.txt");
            let mut file = fs::File::create(&path)?;
            file.write(b"1234567890")?;

            assert_eq!(InputFile::from(io::stdin()).len_hint(), None);
            assert_eq!(InputFile::from(File::open(path)?).len_hint(), Some(10));

            Ok(())
        }
    }
}
