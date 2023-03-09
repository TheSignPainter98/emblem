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
                for (duplicate, original) in dups {
                    ret.push(
                        Log::warn("duplicate attributes")
                            .src({
                                let name = duplicate.name();
                                Src::new(loc)
                                    .annotate(Note::warn(
                                        duplicate.loc(),
                                        format!("found duplicate ‘{}’ here", name),
                                    ))
                                    .annotate(Note::info(
                                        original.loc(),
                                        format!("‘{}’ first defined here", name),
                                    ))
                            })
                            .help("remove multiple occurrences of the same attribute"),
                    );
                }
                ret
            }
            Content::Command { .. }
            | Content::Sugar(_)
            | Content::Word { .. }
            | Content::Whitespace { .. }
            | Content::Dash { .. }
            | Content::Glue { .. }
            | Content::SpiltGlue { .. }
            | Content::Verbatim { .. }
            | Content::Comment { .. }
            | Content::MultiLineComment { .. } => vec![],
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::lint::lints::test::LintTest;

    #[test]
    fn lint() {
        let tests = [
            LintTest {
                lint: DuplicateAttrs::new(),
                num_problems: 0,
                matches: vec![],
                src: "",
            },
            LintTest {
                lint: DuplicateAttrs::new(),
                num_problems: 0,
                matches: vec![],
                src: ".foo",
            },
            LintTest {
                lint: DuplicateAttrs::new(),
                num_problems: 0,
                matches: vec![],
                src: ".foo[]",
            },
            LintTest {
                lint: DuplicateAttrs::new(),
                num_problems: 1,
                matches: vec![
                    ":10-12: found duplicate ‘bar’",
                    ":6-8: ‘bar’ first defined here",
                ],
                src: ".foo[bar,bar]",
            },
            LintTest {
                lint: DuplicateAttrs::new(),
                num_problems: 1,
                matches: vec![
                    ":14-20: found duplicate ‘bar’",
                    ":6-12: ‘bar’ first defined here",
                ],
                src: ".foo[bar=baz,bar=baz]",
            },
            LintTest {
                lint: DuplicateAttrs::new(),
                num_problems: 1,
                matches: vec![
                    ":10-16: found duplicate ‘bar’",
                    ":6-8: ‘bar’ first defined here",
                ],
                src: ".foo[bar,bar=baz]",
            },
            LintTest {
                lint: DuplicateAttrs::new(),
                num_problems: 2,
                matches: vec![
                    ":(10-12|14-16): found duplicate ‘bar’",
                    ":6-8: ‘bar’ first defined here",
                ],
                src: ".foo[bar,bar,bar]",
            },
        ];

        for test in tests {
            test.run();
        }
    }
}
