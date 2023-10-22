use std::collections::HashMap;

use crate::ast::parsed::Attr;
use crate::ast::parsed::Content;
use crate::context::file_content::FileSlice;
use crate::lint::Lint;
use crate::lint::LintId;
use crate::log::{Log, Note, Src};
use derive_new::new;

#[derive(new)]
pub struct DuplicateAttrs {}

impl Lint for DuplicateAttrs {
    fn id(&self) -> LintId {
        "duplicate-attrs".into()
    }

    fn analyse(&mut self, content: &Content) -> Vec<Log> {
        match content {
            Content::Command {
                loc,
                attrs: Some(attrs),
                ..
            } => {
                let mut first_seen: HashMap<&str, &crate::ast::parsed::Attr> = HashMap::new();
                let mut dups: Vec<(_, &Attr)> = vec![];
                for attr in attrs.args() {
                    let repr = attr.repr().to_str();
                    if let Some(first) = first_seen.get(repr) {
                        dups.push((attr, first));
                    } else {
                        first_seen.insert(repr, attr);
                    }
                }

                if dups.is_empty() {
                    return vec![];
                }

                let mut ret = vec![];
                for (duplicate, original) in dups {
                    ret.push(
                        Log::warn("duplicate attributes")
                            .with_src({
                                let repr = duplicate.repr();
                                Src::new(loc)
                                    .with_annotation(Note::warn(
                                        duplicate.loc(),
                                        format!("found duplicate ‘{}’ here", repr),
                                    ))
                                    .with_annotation(Note::info(
                                        original.loc(),
                                        format!("‘{}’ first defined here", repr),
                                    ))
                            })
                            .with_help("remove multiple occurrences of the same attribute"),
                    );
                }
                ret
            }
            Content::Shebang { .. }
            | Content::Command { .. }
            | Content::Sugar(_)
            | Content::Word { .. }
            | Content::Whitespace { .. }
            | Content::Dash { .. }
            | Content::Glue { .. }
            | Content::SpiltGlue { .. }
            | Content::InlineVerbatim { .. }
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
