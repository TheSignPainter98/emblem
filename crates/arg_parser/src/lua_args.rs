use crate::{ext_arg::ExtArg, resource_limit::ResourceLimit, sandbox_level::SandboxLevel};
use clap::{ArgAction::Append, Parser};
use emblem_core::context::{DEFAULT_MAX_MEM, DEFAULT_MAX_STEPS};

/// Holds the user's preferences for the lua environment used when running the program
#[derive(Clone, Debug, Parser, PartialEq, Eq)]
#[warn(missing_docs)]
pub struct LuaArgs {
    /// Pass a named argument into module-space. If module name is omitted, pass argument as
    /// variable in document
    #[arg(short = 'a', action = Append, value_parser = ExtArg::parser(), value_name="mod.arg=value")]
    pub args: Vec<ExtArg>, // TODO(kcza): plumb me!

    /// Limit lua memory usage
    #[arg(long, value_parser = ResourceLimit::<usize>::parser(), default_value_t = ResourceLimit::Limited(DEFAULT_MAX_MEM), value_name = "amount")]
    pub max_mem: ResourceLimit<usize>,

    /// Limit lua execution steps
    #[arg(long, value_parser = ResourceLimit::<u32>::parser(), default_value_t = ResourceLimit::Limited(DEFAULT_MAX_STEPS), value_name = "steps")]
    pub max_steps: ResourceLimit<u32>,

    /// Restrict system access
    #[arg(long = "sandbox", value_enum, default_value_t, value_name = "level")]
    pub sandbox_level: SandboxLevel,
}

impl Default for LuaArgs {
    fn default() -> Self {
        Self {
            args: Default::default(),
            max_mem: ResourceLimit::Limited(DEFAULT_MAX_MEM),
            max_steps: ResourceLimit::Limited(DEFAULT_MAX_STEPS),
            sandbox_level: SandboxLevel::default(),
        }
    }
}
