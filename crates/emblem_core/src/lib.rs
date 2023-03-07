#[macro_use]
pub mod log;

#[cfg(test)]
#[macro_use]
extern crate pretty_assertions;

pub mod args;
pub mod ast;
pub mod build;
pub mod context;
pub mod explain;
pub mod lint;
pub mod parser;
mod path;
mod repo;
mod util;

pub use crate::args::ArgPath;
pub use crate::build::Builder;
pub use crate::context::Context;
pub use crate::explain::Explainer;
pub use crate::lint::Linter;
pub use crate::log::{Log, Verbosity};

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
