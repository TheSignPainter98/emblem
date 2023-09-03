mod lints;

use std::fmt::Display;

use crate::args::ArgPath;
use crate::ast::parsed::{Content, Sugar};
use crate::ast::{File, Par, ParPart};
use crate::context::Context;
use crate::log::messages::Message;
use crate::parser;
use crate::path::SearchResult;
use crate::Action;
use crate::Log;
use crate::{context, EmblemResult};
use derive_more::From;
use derive_new::new;

#[derive(new)]
pub struct Linter {
    input: ArgPath,

    #[allow(unused)]
    fix: bool,
}

impl Action for Linter {
    type Response = ();

    fn run(&self, ctx: &mut context::Context) -> EmblemResult<Self::Response> {
        let problems = match self.input.as_ref().try_into() {
            Ok(r) => self.lint_root(ctx, r),
            Err(e) => vec![Log::error(e.to_string())],
        };
        EmblemResult::new(problems, ())
    }
}

impl Linter {
    fn lint_root(&self, ctx: &mut Context, file: SearchResult) -> Vec<Log> {
        let file = match parser::parse_file(ctx, file) {
            Ok(f) => f,
            Err(e) => return vec![e.log()],
        };

        let mut problems = Vec::new();
        file.lint(&mut lints::lints(), &mut problems);
        problems
    }
}

pub type Lints = Vec<Box<dyn Lint>>;

pub trait Lint {
    fn analyse(&mut self, content: &Content) -> Vec<Log>;

    fn done(&mut self) -> Vec<Log> {
        vec![]
    }

    fn id(&self) -> LintId;
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash, From)]
pub struct LintId(&'static str);

impl LintId {
    pub(crate) fn raw(&self) -> &'static str {
        self.0
    }
}

impl Display for LintId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(f)
    }
}

pub trait Lintable {
    fn lint(&self, lints: &mut Lints, problems: &mut Vec<Log>);
}

impl<T: Lintable> Lintable for File<T> {
    fn lint(&self, lints: &mut Lints, problems: &mut Vec<Log>) {
        self.pars.lint(lints, problems);

        for lint in lints {
            for problem in lint.done() {
                problems.push(problem.with_id(lint.id().into()));
            }
        }
    }
}

impl<T: Lintable> Lintable for Par<T> {
    fn lint(&self, lints: &mut Lints, problems: &mut Vec<Log>) {
        self.parts.lint(lints, problems);
    }
}

impl<T: Lintable> Lintable for ParPart<T> {
    fn lint(&self, lints: &mut Lints, problems: &mut Vec<Log>) {
        match self {
            Self::Command(cmd) => cmd.lint(lints, problems),
            Self::Line(line) => line.lint(lints, problems),
        }
    }
}

impl Lintable for Content {
    fn lint(&self, lints: &mut Lints, problems: &mut Vec<Log>) {
        for lint in lints.iter_mut() {
            for problem in lint.analyse(self) {
                problems.push(problem.with_id(lint.id().into()));
            }
        }

        // Recurse
        match self {
            Self::Command {
                inline_args,
                remainder_arg,
                trailer_args,
                ..
            } => {
                inline_args.lint(lints, problems);
                remainder_arg.lint(lints, problems);
                trailer_args.lint(lints, problems);
            }
            Self::Sugar(sugar) => sugar.lint(lints, problems),
            Self::Shebang { .. }
            | Self::Word { .. }
            | Self::Whitespace { .. }
            | Self::Dash { .. }
            | Self::Glue { .. }
            | Self::SpiltGlue { .. }
            | Self::Verbatim { .. }
            | Self::Comment { .. }
            | Self::MultiLineComment { .. } => {}
        }
    }
}

impl Lintable for Sugar {
    fn lint(&self, lints: &mut Lints, problems: &mut Vec<Log>) {
        match self {
            Self::Italic { arg, .. } => arg.lint(lints, problems),
            Self::Bold { arg, .. } => arg.lint(lints, problems),
            Self::Monospace { arg, .. } => arg.lint(lints, problems),
            Self::Smallcaps { arg, .. } => arg.lint(lints, problems),
            Self::AlternateFace { arg, .. } => arg.lint(lints, problems),
            Self::Heading { arg, .. } => arg.lint(lints, problems),
            Self::Mark { .. } | Self::Reference { .. } => {}
        }
    }
}

impl<T: Lintable> Lintable for Option<T> {
    fn lint(&self, lints: &mut Lints, problems: &mut Vec<Log>) {
        if let Some(t) = self {
            t.lint(lints, problems);
        }
    }
}

impl<T: Lintable> Lintable for Vec<T> {
    fn lint(&self, lints: &mut Lints, problems: &mut Vec<Log>) {
        for elem in self {
            elem.lint(lints, problems)
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn lint_id() {
        let raw = "something-concerning";
        let lint_id = LintId::from(raw);
        assert_eq!(raw, lint_id.raw());
    }
}
