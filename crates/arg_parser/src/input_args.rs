use crate::arg_path::ArgPath;
use clap::{Parser, ValueHint::FilePath};

/// Holds the source of the user's document
#[derive(Clone, Debug, Default, Parser, PartialEq, Eq)]
#[warn(missing_docs)]
pub struct InputArgs {
    /// Document to typeset
    #[arg(value_name = "in-file", value_hint = FilePath, default_value_t = ArgPath::default(), value_parser = ArgPath::parser())]
    pub file: ArgPath,
}
