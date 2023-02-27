#[macro_use]
pub mod log;

#[cfg(test)]
#[macro_use]
extern crate pretty_assertions;

mod args;
mod ast;
pub mod build;
mod context;
pub mod explain;
pub mod init;
pub mod lint;
mod parser;
mod path;
mod repo;
mod util;

pub use args::ArgPath;
pub use build::Builder;
pub use context::Context;
pub use explain::Explainer;
pub use init::Initialiser;
pub use lint::Linter;
pub use log::{Log, Verbosity};

pub trait Action {
    fn run<'em>(&self, ctx: &'em mut context::Context) -> Vec<Log<'em>>;
}
