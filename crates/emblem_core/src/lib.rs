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
mod version;

pub use crate::{
    args::ArgPath,
    build::{
        typesetter::doc::{Doc, DocElem},
        Builder,
    },
    context::Context,
    explain::Explainer,
    lint::Linter,
    log::{Log, Verbosity},
    version::Version,
};

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
