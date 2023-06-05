mod lints;

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
use derive_new::new;

#[derive(new)]
pub struct Linter {
    input: ArgPath,

    #[allow(unused)]
    fix: bool,
}

impl Action for Linter {
    type Response = ();

    fn run<'ctx>(&self, ctx: &'ctx mut context::Context) -> EmblemResult<'ctx, Self::Response> {
        let problems = match self.input.as_ref().try_into() {
            Ok(r) => self.lint_root(ctx, r),
            Err(e) => vec![Log::error(e.to_string())],
        };
        EmblemResult::new(problems, ())
    }
}

impl Linter {
    fn lint_root<'em>(&self, ctx: &'em mut Context, file: SearchResult) -> Vec<Log<'em>> {
        let file = match parser::parse_file(ctx, file) {
            Ok(f) => f,
            Err(e) => return vec![e.log()],
        };

        let mut problems = Vec::new();
        file.lint(&mut lints::lints(), &mut problems);
        problems
    }
}

pub type Lints<'i> = Vec<Box<dyn Lint<'i>>>;

pub trait Lint<'i> {
    fn analyse(&mut self, content: &Content<'i>) -> Vec<Log<'i>>;

    fn done(&mut self) -> Vec<Log<'i>> {
        vec![]
    }

    fn id(&self) -> &'static str;
}

pub trait Lintable<'i> {
    fn lint(&self, lints: &mut Lints<'i>, problems: &mut Vec<Log<'i>>);
}

impl<'i, T: Lintable<'i>> Lintable<'i> for File<T> {
    fn lint(&self, lints: &mut Lints<'i>, problems: &mut Vec<Log<'i>>) {
        self.pars.lint(lints, problems);

        for lint in lints {
            for problem in lint.done() {
                problems.push(problem.with_id(lint.id()));
            }
        }
    }
}

impl<'i, T: Lintable<'i>> Lintable<'i> for Par<T> {
    fn lint(&self, lints: &mut Lints<'i>, problems: &mut Vec<Log<'i>>) {
        self.parts.lint(lints, problems);
    }
}

impl<'i, T: Lintable<'i>> Lintable<'i> for ParPart<T> {
    fn lint(&self, lints: &mut Lints<'i>, problems: &mut Vec<Log<'i>>) {
        match self {
            Self::Command(cmd) => cmd.lint(lints, problems),
            Self::Line(line) => line.lint(lints, problems),
        }
    }
}

impl<'i> Lintable<'i> for Content<'i> {
    fn lint(&self, lints: &mut Lints<'i>, problems: &mut Vec<Log<'i>>) {
        for lint in lints.iter_mut() {
            for problem in lint.analyse(self) {
                problems.push(problem.with_id(lint.id()));
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

impl<'i> Lintable<'i> for Sugar<'i> {
    fn lint(&self, lints: &mut Lints<'i>, problems: &mut Vec<Log<'i>>) {
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

impl<'i, T: Lintable<'i>> Lintable<'i> for Option<T> {
    fn lint(&self, lints: &mut Lints<'i>, problems: &mut Vec<Log<'i>>) {
        if let Some(t) = self {
            t.lint(lints, problems);
        }
    }
}

impl<'i, T: Lintable<'i>> Lintable<'i> for Vec<T> {
    fn lint(&self, lints: &mut Lints<'i>, problems: &mut Vec<Log<'i>>) {
        for elem in self {
            elem.lint(lints, problems)
        }
    }
}
