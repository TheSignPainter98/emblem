use crate::{
    add_cmd::AddCmd, build_cmd::BuildCmd, explain_cmd::ExplainCmd, format_cmd::FormatCmd,
    init_cmd::InitCmd, lint_cmd::LintCmd, list_cmd::ListCmd, lua_args::LuaArgs,
};
use clap::Subcommand;

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
    pub(crate) fn add(&self) -> Option<&AddCmd> {
        match self {
            Self::Add(a) => Some(a),
            _ => None,
        }
    }

    pub(crate) fn build(&self) -> Option<&BuildCmd> {
        match self {
            Self::Build(b) => Some(b),
            _ => None,
        }
    }

    pub(crate) fn explain(&self) -> Option<&ExplainCmd> {
        match self {
            Self::Explain(e) => Some(e),
            _ => None,
        }
    }

    pub(crate) fn format(&self) -> Option<&FormatCmd> {
        match self {
            Self::Format(f) => Some(f),
            _ => None,
        }
    }
    pub(crate) fn init(&self) -> Option<&InitCmd> {
        match self {
            Self::Init(i) => Some(i),
            _ => None,
        }
    }

    pub(crate) fn lint(&self) -> Option<&LintCmd> {
        match self {
            Self::Lint(l) => Some(l),
            _ => None,
        }
    }

    pub(crate) fn list(&self) -> Option<&ListCmd> {
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
