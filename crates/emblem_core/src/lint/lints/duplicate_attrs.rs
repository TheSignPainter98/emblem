use std::collections::HashMap;

use crate::ast::parsed::Attr;
use crate::ast::parsed::Content;
use crate::context::file_content::FileSlice;
use crate::lint::Lint;
use crate::lint::LintId;
use crate::log::{Log, Note, Src};
use crate::Version;
use derive_new::new;

#[derive(Clone, new)]
pub struct DuplicateAttrs {}

impl Lint for DuplicateAttrs {
    fn min_version(&self) -> Version {
        Version::V1_0
    }

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
                        Log::warning("duplicate attributes")
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
        LintTest::new("empty", DuplicateAttrs::new())
            .input("")
            .passes();
        LintTest::new("no-attrs", DuplicateAttrs::new())
            .input(".foo")
            .passes();
        LintTest::new("empty-attrs", DuplicateAttrs::new())
            .input(".foo[]")
            .passes();
        LintTest::new("duplicate-unnamed-attrs", DuplicateAttrs::new())
            .input(".foo[bar,bar]")
            .causes(
                1,
                &[
                    ":10-12: found duplicate ‘bar’",
                    ":6-8: ‘bar’ first defined here",
                ],
            );
        LintTest::new("duplicate-named", DuplicateAttrs::new())
            .input(".foo[bar=baz,bar=baz]")
            .causes(
                1,
                &[
                    ":14-20: found duplicate ‘bar’",
                    ":6-12: ‘bar’ first defined here",
                ],
            );
        LintTest::new("duplicate-key", DuplicateAttrs::new())
            .input(".foo[bar,bar=baz]")
            .causes(
                1,
                &[
                    ":10-16: found duplicate ‘bar’",
                    ":6-8: ‘bar’ first defined here",
                ],
            );
        LintTest::new("duplicate-unnamed", DuplicateAttrs::new())
            .input(".foo[bar,bar,bar]")
            .causes(
                2,
                &[
                    ":(10-12|14-16): found duplicate ‘bar’",
                    ":6-8: ‘bar’ first defined here",
                ],
            );
    }
}
