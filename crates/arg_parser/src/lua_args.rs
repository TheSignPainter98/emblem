use crate::{ext_arg::ExtArg, resource_limit::ResourceLimit, sandbox_level::SandboxLevel};
use clap::{ArgAction::Append, Parser};
use emblem_core::context::{Memory, Step};

/// Holds the user's preferences for the lua environment used when running the program
#[derive(Clone, Debug, Default, Parser, PartialEq, Eq)]
#[warn(missing_docs)]
pub struct LuaArgs {
    /// Pass a named argument into module-space. If module name is omitted, pass argument as
    /// variable in document
    #[arg(short = 'a', action = Append, value_parser = ExtArg::parser(), value_name="mod.arg=value")]
    pub args: Vec<ExtArg>, // TODO(kcza): plumb me!

    /// Limit lua memory usage
    #[arg(long, value_parser = ResourceLimit::<Memory>::parser(), default_value_t, value_name = "amount")]
    pub max_mem: ResourceLimit<Memory>,

    /// Limit lua execution steps
    #[arg(long, value_parser = ResourceLimit::<Step>::parser(), default_value_t, value_name = "steps")]
    pub max_steps: ResourceLimit<Step>,

    /// Restrict system access
    #[arg(long = "sandbox", value_enum, default_value_t, value_name = "level")]
    pub sandbox_level: SandboxLevel,
}
