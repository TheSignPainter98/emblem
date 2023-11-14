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

use crate::lint::Lint;
use crate::lint::Lints;
use crate::Version;

pub fn lints_for(version: Version) -> Lints {
    let lints: [Box<dyn Lint>; 10] = [
        Box::new(attr_ordering::AttrOrdering::new()),
        Box::new(command_naming::CommandNaming::new()),
        Box::new(duplicate_attrs::DuplicateAttrs::new()),
        Box::new(emph_delimiters::EmphDelimiters::new()),
        Box::new(empty_attrs::EmptyAttrs::new()),
        Box::new(num_args::NumArgs::new()),
        Box::new(num_attrs::NumAttrs::new()),
        Box::new(num_pluses::NumPluses::new()),
        Box::new(spilt_glue::SpiltGlue::new()),
        Box::new(sugar_usage::SugarUsage::new()),
    ];
    lints
        .into_iter()
        .filter(|lint| lint.min_version() <= version)
        .collect()
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::{
        lint::{Lint, Lintable},
        log::LogId,
        parser::parse,
        Context,
    };
    use lazy_static::lazy_static;
    use regex::Regex;
    use std::{borrow::Cow, collections::HashSet};

    #[test]
    fn ids() {
        lazy_static! {
            static ref VALID_ID: Regex = Regex::new(r"^[a-z-]+$").unwrap();
        }

        let lints = lints_for(Version::latest());
        let ids = lints.iter().map(|l| l.id()).collect::<Vec<_>>();

        for id in &ids {
            if !VALID_ID.is_match(id.raw()) {
                panic!("IDs should be lowercase with dashes: got {}", id);
            }
        }
    }

    #[test]
    fn unique_ids() {
        let mut ids = HashSet::new();
        for lint in lints_for(Version::latest()) {
            assert!(ids.insert(lint.id()), "id {:?} is not unique", lint.id());
        }
    }

    pub struct LintTest<T>
    where
        T: Lint,
    {
        name: Cow<'static, str>,
        lint: T,
        input: Option<Cow<'static, str>>,
        attempted: bool,
    }

    impl<T> LintTest<T>
    where
        T: Lint + Clone + 'static,
    {
        pub fn new(name: impl Into<Cow<'static, str>>, lint: T) -> Self {
            let name = name.into();
            Self {
                name,
                lint,
                input: None,
                attempted: false,
            }
        }

        pub fn input(mut self, input: impl Into<Cow<'static, str>>) -> Self {
            self.input = Some(input.into());
            self
        }

        pub fn passes(self) {
            self.causes(0, &[]);
        }

        pub fn causes(mut self, expected_problems: u32, matches: &[&str]) {
            self.setup();

            let ctx = Context::test_new();
            let id = self.lint.id();
            let file = parse(
                ctx.alloc_file_name("lint-test.em"),
                ctx.alloc_file_content(self.input.as_ref().unwrap_or(&"".into())),
            )
            .expect("failed to parser output");

            let problems = {
                let mut problems = Vec::new();
                file.lint(&mut vec![Box::new(self.lint.clone())], &mut problems);
                problems
            };
            assert_eq!(
                expected_problems as usize,
                problems.len(),
                "{}: produced {} problems, expected {}",
                self.name,
                problems.len(),
                expected_problems
            );
            for problem in problems {
                problem.assert_compliant();
                assert_eq!(problem.id(), &LogId::from(id), "Incorrect ID");

                let text = problem.annotation_text().join("\n\t");
                for r#match in matches {
                    let re = Regex::new(r#match).unwrap();
                    assert!(
                        re.is_match(&text),
                        "Could not match '{}' in:\n\t{}",
                        &r#match,
                        text
                    );
                }
            }
        }

        fn setup(&mut self) {
            println!("testing {}...", self.name);

            self.attempted = true;

            assert!(self.input.is_some(), "{}: test has no input!", self.name);
        }
    }

    impl<T> Drop for LintTest<T>
    where
        T: Lint,
    {
        fn drop(&mut self) {
            assert!(self.attempted, "test {} never attempted!", self.name);
        }
    }
}
