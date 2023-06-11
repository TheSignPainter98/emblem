mod attr_ordering;
mod command_naming;
mod duplicate_attrs;
mod emph_delimiters;
mod empty_attrs;
mod num_args;
mod num_attrs;
mod num_pluses;
mod spilt_glue;
mod sugar_usage;

use super::Lints;

pub fn lints<'i>() -> Lints<'i> {
    macro_rules! lints {
        ($($lint:expr),* $(,)?) => {
            vec![
                $(Box::new($lint),)*
            ]
        }
    }

    lints![
        attr_ordering::AttrOrdering::new(),
        command_naming::CommandNaming::new(),
        duplicate_attrs::DuplicateAttrs::new(),
        emph_delimiters::EmphDelimiters::new(),
        empty_attrs::EmptyAttrs::new(),
        num_args::NumArgs::new(),
        num_attrs::NumAttrs::new(),
        num_pluses::NumPluses::new(),
        spilt_glue::SpiltGlue::new(),
        sugar_usage::SugarUsage::new(),
    ]
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::{
        lint::{Lint, Lintable},
        parser::parse,
        FileName,
    };
    use lazy_static::lazy_static;
    use regex::Regex;
    use std::collections::HashSet;

    #[test]
    fn ids() {
        lazy_static! {
            static ref VALID_ID: Regex = Regex::new(r"^[a-z-]+$").unwrap();
        }

        let lints = lints();
        let ids = lints.iter().map(|l| l.id()).collect::<Vec<_>>();

        for id in &ids {
            if !VALID_ID.is_match(id) {
                panic!("IDs should be lowercase with dashes: got {}", id);
            }
        }
    }

    #[test]
    fn unique_ids() {
        let mut ids = HashSet::new();
        for lint in lints() {
            assert!(ids.insert(lint.id()), "id {:?} is not unique", lint.id());
        }
    }

    pub struct LintTest<'i, L>
    where
        L: Lint<'i> + 'static,
    {
        pub lint: L,
        pub num_problems: usize,
        pub matches: Vec<&'i str>,
        pub src: &'i str,
    }

    impl<'i, L> LintTest<'i, L>
    where
        L: Lint<'i> + 'static,
    {
        pub fn run(self) {
            let id = self.lint.id();
            let file =
                parse(FileName::new("lint-test.em"), self.src).expect("Failed to parse input");

            let problems = {
                let mut problems = Vec::new();
                file.lint(&mut vec![Box::new(self.lint)], &mut problems);
                problems
            };

            assert_eq!(
                self.num_problems,
                problems.len(),
                "{} problems testing {} (expected {})",
                problems.len(),
                self.src,
                self.num_problems
            );

            for problem in problems {
                problem.assert_compliant();

                assert_eq!(problem.id(), Some(id), "Incorrect ID");

                let text = problem.annotation_text().join("\n\t");
                for r#match in &self.matches {
                    let re = Regex::new(r#match).unwrap();
                    assert!(
                        re.is_match(&text),
                        "Could not match '{}' in:\n\t{}",
                        r#match,
                        text
                    );
                }
            }
        }
    }
}
