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
mod extensions;
pub mod lint;
pub mod parser;
mod path;
mod repo;
mod util;
mod version;

pub use crate::{
    args::ArgPath,
    build::{
        typesetter::{
            doc::{Doc, DocElem},
            Typesetter,
        },
        Builder,
    },
    context::{
        file_content::{FileContent, FileContentSlice},
        file_name::FileName,
        Context, ResourceLimit, SandboxLevel,
    },
    explain::Explainer,
    extensions::ExtensionState,
    lint::Linter,
    log::{Log, Verbosity},
    version::Version,
};

use derive_new::new;

pub trait Action {
    type Response;

    fn run(&self, ctx: &mut Context) -> EmblemResult<Self::Response>;

    fn output(&self, _: Self::Response) -> EmblemResult<()> {
        EmblemResult::new(vec![], ())
    }
}

#[derive(new, Debug)]
pub struct EmblemResult<R> {
    pub logs: Vec<Log>,
    pub response: R,
}

impl<T> EmblemResult<T> {
    pub fn successful(&self, warnings_as_errors: bool) -> bool {
        self.logs.iter().all(|l| l.successful(warnings_as_errors))
    }
}
