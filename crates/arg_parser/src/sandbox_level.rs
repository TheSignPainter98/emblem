use clap::ValueEnum;
use emblem_core::context::SandboxLevel as EmblemSandboxLevel;

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
