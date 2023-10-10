#[macro_use]
pub mod log;

#[cfg(test)]
#[macro_use]
extern crate pretty_assertions;

pub mod args;
pub mod ast;
pub mod build;
pub mod context;
mod error;
pub mod explain;
mod extensions;
pub mod lint;
pub mod parser;
mod path;
mod repo;
mod result;
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
    error::Error,
    explain::Explainer,
    extensions::ExtensionState,
    lint::Linter,
    log::{Log, Verbosity},
    result::{ErrorContext, Result},
    version::Version,
};

pub trait Action {
    type Response;

    fn run(&self, ctx: &mut Context) -> Result<Self::Response>;
}
