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

use derive_new::new;

pub trait Action {
    type Response;

    fn run<'ctx>(&self, ctx: &'ctx mut context::Context) -> EmblemResult<'ctx, Self::Response>;

    fn output<'ctx>(&self, _: Self::Response) -> EmblemResult<'ctx, ()> {
        EmblemResult::new(vec![], ())
    }
}

#[derive(new, Debug)]
pub struct EmblemResult<'em, R> {
    pub logs: Vec<Log<'em>>,
    pub response: R,
}

impl<'em, T> EmblemResult<'em, T> {
    pub fn successful(&self, warnings_as_errors: bool) -> bool {
        self.logs.iter().all(|l| l.successful(warnings_as_errors))
    }
}
