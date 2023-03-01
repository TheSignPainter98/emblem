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
use std::io;

pub trait Action {
    fn run<'ctx>(&self, ctx: &'ctx mut context::Context) -> EmblemResult<'ctx>;
}

#[derive(new, Debug)]
pub struct EmblemResult<'em> {
    pub logs: Vec<Log<'em>>,
    pub result: ActionResult,
}

impl<'em> EmblemResult<'em> {
    pub fn successful(&self, warnings_as_errors: bool) -> bool {
        self.logs.iter().all(|l| l.successful(warnings_as_errors))
    }
}

#[non_exhaustive]
#[derive(Debug, PartialEq)]
pub enum ActionResult {
    Build {
        output_files: Option<Vec<(ArgPath, String)>>,
    },
    Explain {
        explanation: Option<&'static str>,
    },
    // Format,
    Init,
    Lint,
    // List,
}

impl ActionResult {
    pub fn output(&self) -> io::Result<()> {
        match self {
            Self::Build { output_files } => {
                if let Some(output_files) = output_files {
                    for (path, content) in output_files {
                        match path {
                            ArgPath::Stdio => println!("{}", content),
                            ArgPath::Path(p) => std::fs::write(p, content)?,
                        }
                    }
                }
            }
            Self::Explain { explanation } => {
                if let Some(expl) = explanation {
                    println!("{}", expl);
                }
            }
            Self::Init | Self::Lint => {}
        }
        Ok(())
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn successful() {
        for warnings_as_errors in [false, true] {
            let r = EmblemResult::new(vec![], ActionResult::Init);
            assert!(r.successful(warnings_as_errors));
        }

        for warnings_as_errors in [false, true] {
            let r = EmblemResult::new(vec![Log::error("foo")], ActionResult::Init);
            assert!(!r.successful(warnings_as_errors));
        }

        for warnings_as_errors in [false, true] {
            let r = EmblemResult::new(vec![Log::warn("foo")], ActionResult::Init);
            assert_eq!(r.successful(warnings_as_errors), !warnings_as_errors);
        }

        for warnings_as_errors in [false, true] {
            let r = EmblemResult::new(vec![Log::info("foo")], ActionResult::Init);
            assert!(r.successful(warnings_as_errors));
        }
    }
}
