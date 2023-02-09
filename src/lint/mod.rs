mod lints;

use crate::args::LintCmd;
use crate::args::SearchResult;
use crate::ast::parsed::Content;
use crate::ast::{File, Par, ParPart};
use crate::context::Context;
use crate::log::Log;
use crate::parser::{self, Error as ParseError};
use std::error::Error;

pub fn lint(cmd: LintCmd) -> Result<(), Box<dyn Error>> {
    let mut ctx = Context::new();

    match lint_root(&mut ctx, cmd.input.file.as_ref().try_into()?) {
        Ok(problems) => {
            for problem in problems.into_iter() {
                alert!(problem);
            }
        }
        Err(e) => alert!(e),
    }

    Ok(())
}

fn lint_root<'i>(
    ctx: &'i mut Context,
    file: SearchResult,
) -> Result<Vec<Log<'i>>, Box<ParseError<'i>>> {
    let file = parser::parse_file(ctx, file)?;

    let mut problems = Vec::new();
    file.lint(&mut lints::lints(), &mut problems);
    Ok(problems)
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
                problems.push(problem.id(lint.id()));
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
                problems.push(problem.id(lint.id()));
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
            Self::Word { .. }
            | Self::Whitespace { .. }
            | Self::Dash { .. }
            | Self::Glue { .. }
            | Self::Verbatim { .. }
            | Self::Comment { .. }
            | Self::MultiLineComment { .. } => {}
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
