mod lints;
mod problem;

use crate::args::LintCmd;
use crate::ast::parsed::Content;
use crate::ast::parsed::ParsedFile;
use crate::ast::{File, Par, ParPart};
use crate::context::Context;
use crate::parser;
use problem::Problem;
use std::error::Error;

pub fn lint(cmd: LintCmd) -> Result<(), Box<dyn Error>> {
    let mut ctx = Context::new();

    let file: ParsedFile = match parser::parse_file(&mut ctx, cmd.input.file.as_ref().try_into()?) {
        Ok(f) => f,
        Err(e) => {
            println!("{}", e);
            return Ok(());
        }
    };

    let mut lints = lints::lints();
    let mut problems = Vec::new();

    file.lint(&mut lints, &mut problems);

    for problem in &problems {
        println!("{}", problem);
    }

    let len = problems.len();
    if len > 0 {
        println!("{} linting problems", problems.len());
    }

    Ok(())
}

pub type Lints = Vec<Box<dyn Lint>>;

pub trait Lint {
    fn analyse<'i>(&mut self, content: &Content<'i>) -> Option<Problem>;

    fn done(&mut self) -> Option<Problem> { None }

    fn id(&self) -> &'static str;

    fn problem(&self, reason: String) -> Problem {
        Problem::new(self.id(), reason)
    }
}

pub trait Lintable {
    fn lint(&self, lints: &mut Lints, problems: &mut Vec<Problem>);
}

impl<T: Lintable> Lintable for File<T> {
    fn lint(&self, lints: &mut Lints, problems: &mut Vec<Problem>) {
        self.pars.lint(lints, problems);

        for lint in lints {
            if let Some(problem) = lint.done() {
                problems.push(problem);
            }
        }
    }
}

impl<T: Lintable> Lintable for Par<T> {
    fn lint(&self, lints: &mut Lints, problems: &mut Vec<Problem>) {
        self.parts.lint(lints, problems);
    }
}

impl<T: Lintable> Lintable for ParPart<T> {
    fn lint(&self, lints: &mut Lints, problems: &mut Vec<Problem>) {
        match self {
            Self::Command(cmd) => cmd.lint(lints, problems),
            Self::Line(line) => line.lint(lints, problems),
        }
    }
}

impl Lintable for Content<'_> {
    fn lint(&self, lints: &mut Lints, problems: &mut Vec<Problem>) {
        for lint in lints.iter_mut() {
            if let Some(problem) = lint.analyse(self) {
                problems.push(problem);
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
                if let Some(arg) = remainder_arg {
                    arg.lint(lints, problems);
                }
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

impl<T: Lintable> Lintable for Vec<T> {
    fn lint(&self, lints: &mut Lints, problems: &mut Vec<Problem>) {
        for elem in self {
            elem.lint(lints, problems)
        }
    }
}
