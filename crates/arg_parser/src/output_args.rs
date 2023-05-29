use crate::arg_path::UninferredArgPath;
use clap::{Parser, ValueHint::AnyPath};

/// Holds where and how the user wants their output
#[derive(Clone, Debug, Default, Parser, PartialEq, Eq)]
#[warn(missing_docs)]
pub struct OutputArgs {
    /// Output file path
    #[arg(value_name = "out-file", value_hint = AnyPath, default_value_t=UninferredArgPath::default(), value_parser = UninferredArgPath::parser())]
    pub(crate) stem: UninferredArgPath,

    /// Override detected output format
    #[arg(short = 'T', value_name = "format")]
    pub driver: Option<String>,
}
