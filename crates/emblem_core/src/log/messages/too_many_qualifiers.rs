use crate::log::messages::Message;
use crate::log::{Log, Note, Src};
use crate::parser::Location;
use crate::util;
use derive_new::new;
use indoc::indoc;

#[derive(Default, new)]
pub struct TooManyQualifiers<'i> {
    loc: Location<'i>,
    dot_locs: Vec<Location<'i>>,
}

impl<'i> Message<'i> for TooManyQualifiers<'i> {
    fn id() -> &'static str {
        "E005"
    }

    fn log(self) -> Log<'i> {
        Log::error(format!(
            "too many {} found in call",
            util::plural(self.dot_locs.len(), "dot", "dots")
        ))
        .with_id(Self::id())
        .explainable()
        .with_src({
            let mut src = Src::new(&self.loc);
            for (i, dot_loc) in self.dot_locs.iter().enumerate() {
                src = src.with_annotation(Note::error(
                    dot_loc,
                    match i {
                        0 => "found here",
                        _ => "and here",
                    },
                ));
            }
            src
        })
    }

    fn explain(&self) -> &'static str {
        indoc!("
            This error means that a command has been specified with too many qualifiers, for
            example, '.foo.bar.baz.quz'. To keep things concise, emblem allows at most one
            qualifier per command, for example '.foo.bar'.
        ")
    }
}
