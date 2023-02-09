use std::collections::HashMap;

use crate::ast::parsed::Attr;
use crate::ast::parsed::Content;
use crate::lint::Lint;
use crate::log::{Log, Note, Src};
use derive_new::new;

#[derive(new)]
pub struct DuplicateAttrs {}

impl<'i> Lint<'i> for DuplicateAttrs {
    fn id(&self) -> &'static str {
        "duplicate-attrs"
    }

    fn analyse(&mut self, content: &Content<'i>) -> Vec<Log<'i>> {
        match content {
            Content::Command {
                loc,
                attrs: Some(attrs),
                ..
            } => {
                let mut first_seen: HashMap<&str, &crate::ast::parsed::Attr> = HashMap::new();
                let mut dups = Vec::new();
                for attr in attrs.args() {
                    let name = attr.name();
                    if let Some(first) = first_seen.get(name) {
                        dups.push((attr, <&Attr<'_>>::clone(first)));
                    } else {
                        first_seen.insert(name, attr);
                    }
                }

                if dups.is_empty() {
                    return vec![];
                }

                let mut ret = vec![];
                for (dup, def) in dups {
                    ret.push(
                        Log::warn("duplicate attributes")
                            .src({
                                let name = dup.name();
                                Src::new(loc)
                                    .annotate(Note::warn(
                                        dup.loc(),
                                        format!("found duplicate '{}' here", name),
                                    ))
                                    .annotate(Note::info(
                                        def.loc(),
                                        format!("'{}' first defined here", name),
                                    ))
                            })
                            .help("remove multiple occurrences of the same attribute"),
                    );
                }
                ret
            }
            Content::Command { .. }
            | Content::Word { .. }
            | Content::Whitespace { .. }
            | Content::Dash { .. }
            | Content::Glue { .. }
            | Content::Verbatim { .. }
            | Content::Comment { .. }
            | Content::MultiLineComment { .. } => vec![],
        }
    }
}

#[cfg(test)]
mod test {
    use regex::Regex;

    use super::*;
    use crate::{lint::Lintable, parser::parse};
    use std::error::Error;

    struct LintTest<'i, L: Lint<'i> + 'static> {
        lint: L,
        num_problems: usize,
        matches: &'i [&'i str],
        src: &'i str,
    }

    impl<'i, L: Lint<'i> + 'static> LintTest<'i, L> {
        pub fn run(self) {
            let file = parse("lint-test.em", self.src).expect("Failed to parse input");

            let problems = {
                let mut problems = Vec::new();
                file.lint(&mut vec![Box::new(self.lint)], &mut problems);
                problems
            };

            assert_eq!(self.num_problems, problems.len());

            for problem in problems {
                let text = problem.get_annotation_text().join("\n\t");
                for r#match in self.matches {
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

    #[test]
    fn lint() -> Result<(), Box<dyn Error>> {
        let tests = [
            LintTest {
                lint: DuplicateAttrs::new(),
                num_problems: 1,
                matches: &[
                    ":10-12: found duplicate 'bar'",
                    ":6-8: 'bar' first defined here",
                ],
                src: ".foo[bar,bar]",
            },
            LintTest {
                lint: DuplicateAttrs::new(),
                num_problems: 1,
                matches: &[
                    ":14-20: found duplicate 'bar'",
                    ":6-12: 'bar' first defined here",
                ],
                src: ".foo[bar=baz,bar=baz]",
            },
            LintTest {
                lint: DuplicateAttrs::new(),
                num_problems: 1,
                matches: &[
                    ":10-16: found duplicate 'bar'",
                    ":6-8: 'bar' first defined here",
                ],
                src: ".foo[bar,bar=baz]",
            },
        ];

        for test in tests {
            test.run();
        }

        Ok(())
    }
}
