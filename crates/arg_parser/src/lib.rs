use clap::{
    builder::{OsStr, StringValueParser, TypedValueParser},
    error,
    ArgAction::{Append, Count, Help, Version},
    CommandFactory, Parser, Subcommand, ValueEnum,
    ValueHint::{AnyPath, DirPath, FilePath},
};
use emblem_core::context::{
    ResourceLimit as EmblemResourceLimit, SandboxLevel as EmblemSandboxLevel,
};
use num::{Bounded, FromPrimitive, Integer, ToPrimitive};
use std::{env, ffi::OsString, fmt::Display, path, str::FromStr};

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

const LONG_ABOUT: &str = "Takes input of a markdown-like document, processes it and typesets it before passing the result to a driver for outputting in some format. Extensions can be used to include arbitrary functionality; device drivers can be defined by extensions.";

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
    /// Add an extension the current document's compilation
    Add(AddCmd),

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

impl Command {
    pub fn lua_args(&self) -> Option<&LuaArgs> {
        match self {
            Self::Add(_) => None,
            Self::Build(cmd) => Some(&cmd.lua),
            Self::Explain(_) => None,
            Self::Format(_) => None,
            Self::Init(_) => None,
            Self::Lint(cmd) => Some(&cmd.lua),
            Self::List(cmd) => Some(&cmd.lua),
        }
    }
}

#[cfg(test)]
impl Command {
    fn add(&self) -> Option<&AddCmd> {
        match self {
            Self::Add(a) => Some(a),
            _ => None,
        }
    }

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

/// Arguments to the build subcommand
#[derive(Clone, Debug, Parser, PartialEq, Eq)]
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
    pub lua: LuaArgs,

    /// Max iterations of the typesetting loop
    #[arg(long, value_parser = ResourceLimit::<u32>::parser(), default_value_t = ResourceLimit::Limited(5), value_name = "max")]
    pub max_iters: ResourceLimit<u32>,
}

impl Default for BuildCmd {
    fn default() -> Self {
        Self {
            input: Default::default(),
            output: Default::default(),
            lua: Default::default(),
            max_iters: ResourceLimit::Limited(5),
        }
    }
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, Default)]
pub enum ResourceLimit<T: Bounded + Clone + Copy + Integer> {
    #[default]
    Unlimited,
    Limited(T),
}

impl<T> ResourceLimit<T>
where
    T: Bounded
        + Clone
        + Copy
        + Display
        + FromPrimitive
        + FromStr
        + Integer
        + ToPrimitive
        + Send
        + Sync
        + 'static,
    <T as FromStr>::Err: Display,
{
    fn parser() -> impl TypedValueParser {
        StringValueParser::new().try_map(Self::try_from)
    }
}

impl From<ResourceLimit<u32>> for u32 {
    fn from(max: ResourceLimit<u32>) -> Self {
        match max {
            ResourceLimit::Unlimited => u32::max_value(),
            ResourceLimit::Limited(l) => l,
        }
    }
}

impl From<ResourceLimit<usize>> for usize {
    fn from(max: ResourceLimit<usize>) -> Self {
        match max {
            ResourceLimit::Unlimited => usize::max_value(),
            ResourceLimit::Limited(l) => l,
        }
    }
}

impl<T> TryFrom<String> for ResourceLimit<T>
where
    T: Bounded
        + Clone
        + Copy
        + Display
        + FromPrimitive
        + FromStr
        + Integer
        + ToPrimitive
        + Send
        + Sync
        + 'static,
    <T as FromStr>::Err: Display,
{
    type Error = error::Error;

    fn try_from(raw: String) -> Result<Self, Self::Error> {
        Self::try_from(&raw[..])
    }
}

impl<T> TryFrom<&str> for ResourceLimit<T>
where
    T: Bounded
        + Clone
        + Copy
        + Display
        + FromPrimitive
        + FromStr
        + Integer
        + ToPrimitive
        + Send
        + Sync
        + 'static,
    <T as FromStr>::Err: Display,
{
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

        let amt: T = match raw_amt.parse() {
            Ok(a) => a,
            Err(e) => {
                let mut cmd = RawArgs::command();
                return Err(cmd.error(error::ErrorKind::InvalidValue, e));
            }
        };

        let multiplier: T = {
            let max = T::max_value()
                .to_usize()
                .expect("internal error: max value too large");
            match &unit[..] {
                "K" if max >= 1 << 10 => T::from_u32(1 << 10).unwrap(),
                "M" if max >= 1 << 20 => T::from_u32(1 << 20).unwrap(),
                "G" if max >= 1 << 30 => T::from_u32(1 << 30).unwrap(),
                "" => T::from_u32(1).unwrap(),
                _ => {
                    let mut cmd = RawArgs::command();
                    return Err(cmd.error(
                        error::ErrorKind::InvalidValue,
                        format!("unrecognised unit: {}", unit),
                    ));
                }
            }
        };

        if T::max_value().to_f64().unwrap() < multiplier.to_f64().unwrap() * amt.to_f64().unwrap() {
            let mut cmd = RawArgs::command();
            return Err(cmd.error(
                error::ErrorKind::InvalidValue,
                format!(
                    "resource limit too large, expected at most {}",
                    T::max_value()
                ),
            ));
        }

        Ok(Self::Limited(amt * multiplier))
    }
}

impl<T> Display for ResourceLimit<T>
where
    T: Bounded + Clone + Copy + Display + Integer,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Unlimited => write!(f, "unlimited"),
            Self::Limited(l) => write!(f, "{l}"),
        }
    }
}

impl BuildCmd {
    #[allow(dead_code)]
    pub fn output_stem(&self) -> ArgPath {
        self.output.stem.infer_from(&self.input.file)
    }
}

impl From<&BuildCmd> for emblem_core::Builder {
    fn from(cmd: &BuildCmd) -> Self {
        let output_stem = cmd.output_stem().into();
        emblem_core::Builder::new(
            cmd.input.file.clone().into(),
            output_stem,
            cmd.output.driver.clone(),
            cmd.max_iters.into(),
            cmd.lua.clone().into(),
        )
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

impl From<&ExplainCmd> for emblem_core::Explainer {
    fn from(cmd: &ExplainCmd) -> Self {
        Self::new(cmd.id.clone())
    }
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
    pub dir: String,
}

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

impl From<&LintCmd> for emblem_core::Linter {
    fn from(cmd: &LintCmd) -> Self {
        Self::new(cmd.input.file.clone().into(), cmd.fix)
    }
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
    pub lua: LuaArgs,
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

/// Holds the user's preferences for the lua environment used when running the program
#[derive(Clone, Debug, Default, Parser, PartialEq, Eq)]
#[warn(missing_docs)]
pub struct LuaArgs {
    /// Pass a named argument into module-space. If module name is omitted, pass argument as
    /// variable in document
    #[arg(short = 'a', action = Append, value_parser = ExtArg::parser(), value_name="mod.arg=value")]
    pub args: Vec<ExtArg>, // TODO(kcza): plumb me!

    /// Limit lua memory usage
    #[arg(long, value_parser = ResourceLimit::<usize>::parser(), default_value = "unlimited", value_name = "amount")]
    pub max_mem: ResourceLimit<usize>,

    /// Limit lua execution steps
    #[arg(long, value_parser = ResourceLimit::<u32>::parser(), default_value = "unlimited", value_name = "steps")]
    pub max_steps: ResourceLimit<u32>,

    /// Restrict system access
    #[arg(long, value_enum, default_value_t, value_name = "level")]
    pub sandbox: SandboxLevel,
}

impl From<LuaArgs> for emblem_core::ExtensionStateBuilder {
    fn from(args: LuaArgs) -> Self {
        Self {
            sandbox_level: args.sandbox.into(),
            max_steps: args.max_steps.into(),
            max_mem: args.max_mem.into(),
        }
    }
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
}

impl Default for ArgPath {
    fn default() -> Self {
        Self::Path("main.em".into())
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

impl From<ArgPath> for emblem_core::ArgPath {
    fn from(path: ArgPath) -> Self {
        match path {
            ArgPath::Stdio => Self::Stdio,
            ArgPath::Path(p) => Self::Path(p),
        }
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

impl<T> From<ResourceLimit<T>> for EmblemResourceLimit<T>
where
    T: Bounded + Integer + Clone + Copy,
{
    fn from(limit: ResourceLimit<T>) -> Self {
        match limit {
            ResourceLimit::Limited(n) => Self::Limited(n),
            ResourceLimit::Unlimited => Self::Unlimited,
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

impl From<Verbosity> for emblem_core::Verbosity {
    fn from(v: Verbosity) -> Self {
        match v {
            Verbosity::Terse => Self::Terse,
            Verbosity::Verbose => Self::Verbose,
            Verbosity::Debug => Self::Debug,
        }
    }
}

#[derive(ValueEnum, Copy, Clone, Debug, Default, PartialEq, Eq)]
pub enum SandboxLevel {
    /// Place no restrictions on the Lua environment.
    Unrestricted,

    /// Prohibit subprocesses creation and file system access outside of the current
    /// document's directory.
    #[default]
    Standard,

    /// Same restrictions as Standard, but all file system access is prohibited.
    Strict,
}

impl From<SandboxLevel> for EmblemSandboxLevel {
    fn from(level: SandboxLevel) -> Self {
        match level {
            SandboxLevel::Unrestricted => Self::Unrestricted,
            SandboxLevel::Standard => Self::Standard,
            SandboxLevel::Strict => Self::Strict,
        }
    }
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
                Args::try_parse_from(["em"]).unwrap().command,
                Args::try_parse_from(["em", "build"]).unwrap().command
            );
        }

        mod common {
            use super::*;

            #[test]
            fn colourise_output() {
                assert!(
                    !Args::try_parse_from(["em", "--colour", "never"])
                        .unwrap()
                        .log
                        .colour
                );
                assert!(
                    Args::try_parse_from(["em", "--colour", "always"])
                        .unwrap()
                        .log
                        .colour
                );

                assert!(Args::try_parse_from(["em", "--colour", "crabcakes"]).is_err());
            }

            #[test]
            fn warnings_as_errors() {
                assert!(!Args::try_parse_from(["em"]).unwrap().log.warnings_as_errors);
                assert!(
                    Args::try_parse_from(["em", "-E"])
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

        mod add {
            use super::*;

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
                assert!(Args::try_parse_from([
                    "em", "add", "pootis", "--commit", "COMMIT", "--tag", "TAG"
                ])
                .is_err());
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

        mod build {
            use super::*;

            #[test]
            fn output_driver() {
                assert_eq!(
                    Args::try_parse_from(["em"])
                        .unwrap()
                        .command
                        .build()
                        .unwrap()
                        .output
                        .driver,
                    None
                );
                assert_eq!(
                    Args::try_parse_from(["em", "build", "-T", "pies"])
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
                    Args::try_parse_from(["em"])
                        .unwrap()
                        .command
                        .build()
                        .unwrap()
                        .input
                        .file,
                    ArgPath::try_from("main.em").unwrap(),
                );
                assert_eq!(
                    Args::try_parse_from(["em", "build", "-"])
                        .unwrap()
                        .command
                        .build()
                        .unwrap()
                        .input
                        .file,
                    ArgPath::Stdio
                );
                assert_eq!(
                    Args::try_parse_from(["em", "build", "chickens"])
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
                    Args::try_parse_from(["em"])
                        .unwrap()
                        .command
                        .build()
                        .unwrap()
                        .output_stem(),
                    ArgPath::try_from("main.em").unwrap(),
                );
                assert_eq!(
                    Args::try_parse_from(["em", "build", "-"])
                        .unwrap()
                        .command
                        .build()
                        .unwrap()
                        .output_stem(),
                    ArgPath::Stdio,
                );
                assert_eq!(
                    Args::try_parse_from(["em", "build", "-", "pies"])
                        .unwrap()
                        .command
                        .build()
                        .unwrap()
                        .output_stem(),
                    ArgPath::try_from("pies").unwrap(),
                );
                assert_eq!(
                    Args::try_parse_from(["em", "build", "_", "-"])
                        .unwrap()
                        .command
                        .build()
                        .unwrap()
                        .output_stem(),
                    ArgPath::Stdio,
                );
                assert_eq!(
                    Args::try_parse_from(["em", "build", "_", "pies"])
                        .unwrap()
                        .command
                        .build()
                        .unwrap()
                        .output_stem(),
                    ArgPath::try_from("pies").unwrap(),
                );
                assert_eq!(
                    Args::try_parse_from(["em", "build", "-", "pies"])
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
                    Args::try_parse_from(["em"])
                        .unwrap()
                        .command
                        .build()
                        .unwrap()
                        .lua
                        .max_mem,
                    ResourceLimit::Unlimited
                );
                assert_eq!(
                    Args::try_parse_from(["em", "build", "--max-mem", "25"])
                        .unwrap()
                        .command
                        .build()
                        .unwrap()
                        .lua
                        .max_mem,
                    ResourceLimit::Limited(25)
                );
                assert_eq!(
                    Args::try_parse_from(["em", "build", "--max-mem", "25K"])
                        .unwrap()
                        .command
                        .build()
                        .unwrap()
                        .lua
                        .max_mem,
                    ResourceLimit::Limited(25 * 1024)
                );
                assert_eq!(
                    Args::try_parse_from(["em", "build", "--max-mem", "25M"])
                        .unwrap()
                        .command
                        .build()
                        .unwrap()
                        .lua
                        .max_mem,
                    ResourceLimit::Limited(25 * 1024 * 1024)
                );
                assert_eq!(
                    Args::try_parse_from(["em", "build", "--max-mem", "25G"])
                        .unwrap()
                        .command
                        .build()
                        .unwrap()
                        .lua
                        .max_mem,
                    ResourceLimit::Limited(25 * 1024 * 1024 * 1024)
                );

                assert!(Args::try_parse_from(["em", "build", "--max-mem", "100T"]).is_err());
            }

            #[test]
            fn max_steps() {
                assert_eq!(
                    Args::try_parse_from(["em"])
                        .unwrap()
                        .command
                        .build()
                        .unwrap()
                        .lua
                        .max_steps,
                    ResourceLimit::Unlimited
                );
                assert_eq!(
                    Args::try_parse_from(["em", "build", "--max-steps", "25"])
                        .unwrap()
                        .command
                        .build()
                        .unwrap()
                        .lua
                        .max_steps,
                    ResourceLimit::Limited(25)
                );
                assert_eq!(
                    Args::try_parse_from(["em", "build", "--max-steps", "25K"])
                        .unwrap()
                        .command
                        .build()
                        .unwrap()
                        .lua
                        .max_steps,
                    ResourceLimit::Limited(25 * 1024)
                );
                assert_eq!(
                    Args::try_parse_from(["em", "build", "--max-steps", "25M"])
                        .unwrap()
                        .command
                        .build()
                        .unwrap()
                        .lua
                        .max_steps,
                    ResourceLimit::Limited(25 * 1024 * 1024)
                );

                assert!(Args::try_parse_from(["em", "build", "--max-steps", "100T"]).is_err());
            }

            #[test]
            fn sandbox() {
                assert_eq!(
                    Args::try_parse_from(["em"])
                        .unwrap()
                        .command
                        .build()
                        .unwrap()
                        .lua
                        .sandbox,
                    SandboxLevel::Standard
                );
                assert_eq!(
                    Args::try_parse_from(["em", "build", "--sandbox", "unrestricted"])
                        .unwrap()
                        .command
                        .build()
                        .unwrap()
                        .lua
                        .sandbox,
                    SandboxLevel::Unrestricted
                );
                assert_eq!(
                    Args::try_parse_from(["em", "build", "--sandbox", "standard"])
                        .unwrap()
                        .command
                        .build()
                        .unwrap()
                        .lua
                        .sandbox,
                    SandboxLevel::Standard
                );
                assert_eq!(
                    Args::try_parse_from(["em", "build", "--sandbox", "strict"])
                        .unwrap()
                        .command
                        .build()
                        .unwrap()
                        .lua
                        .sandbox,
                    SandboxLevel::Strict
                );

                assert!(Args::try_parse_from(["em", "build", "--sandbox", "root"]).is_err());
            }

            #[test]
            fn module_args() {
                assert_eq!(
                    Args::try_parse_from(["em"])
                        .unwrap()
                        .command
                        .build()
                        .unwrap()
                        .lua
                        .args,
                    vec![]
                );

                {
                    let valid_ext_args =
                        Args::try_parse_from(["em", "build", "-ak=v", "-ak2=v2", "-ak3="])
                            .unwrap()
                            .command
                            .build()
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

                assert!(Args::try_parse_from(["em", "-a=v"]).is_err());
            }

            #[test]
            fn max_iters() {
                assert_eq!(
                    Args::try_parse_from(["em", "build"])
                        .unwrap()
                        .command
                        .build()
                        .unwrap()
                        .max_iters,
                    ResourceLimit::Limited(5),
                );
                assert_eq!(
                    Args::try_parse_from(["em", "build", "--max-iters", "25"])
                        .unwrap()
                        .command
                        .build()
                        .unwrap()
                        .max_iters,
                    ResourceLimit::Limited(25),
                );
                assert_eq!(
                    Args::try_parse_from(["em", "build", "--max-iters", "unlimited"])
                        .unwrap()
                        .command
                        .build()
                        .unwrap()
                        .max_iters,
                    ResourceLimit::Unlimited,
                );
            }
        }

        mod explain {
            use super::*;

            #[test]
            fn code() {
                assert_eq!(
                    Args::try_parse_from(["em", "explain", "E001"])
                        .unwrap()
                        .command
                        .explain()
                        .unwrap()
                        .id,
                    "E001"
                );
                assert!(Args::try_parse_from(["em", "explain"]).is_err());
            }
        }

        mod format {
            use super::*;

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

        mod init {
            use super::*;

            #[test]
            fn dir() {
                assert_eq!(
                    Args::try_parse_from(["em", "init"])
                        .unwrap()
                        .command
                        .init()
                        .unwrap()
                        .dir,
                    ".",
                );
                assert_eq!(
                    Args::try_parse_from(["em", "init", "cool-doc"])
                        .unwrap()
                        .command
                        .init()
                        .unwrap()
                        .dir,
                    "cool-doc",
                );
            }
        }

        mod lint {
            use super::*;

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
                    let valid_ext_args =
                        Args::try_parse_from(["em", "lint", "-ak=v", "-ak2=v2", "-ak3="])
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

        mod list {
            use super::*;

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
                    let valid_ext_args = Args::try_parse_from([
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
    }
}
