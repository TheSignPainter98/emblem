use crate::log::messages::Message;
use crate::log::{Log, LogId, Note, Src};
use crate::parser::Location;
use derive_new::new;
use indoc::indoc;

#[derive(Default, new)]
pub struct EmptyQualifier {
    loc: Location,
    qualifier_loc: Location,
}

impl Message for EmptyQualifier {
    fn id() -> LogId {
        "E004".into()
    }

    fn log(self) -> Log {
        Log::error("empty qualifier in command name")
            .with_id(Self::id())
            .explainable()
            .with_src(
                Src::new(&self.loc).with_annotation(Note::error(&self.qualifier_loc, "found here")),
            )
    }

    fn explain(&self) -> &'static str {
        indoc! {"
            This error means that a command has been called with an empty qualifier. This likely
            originates from a call which looks like `..cmd`. There are two ways to call a command:

            .cmd     // qualifier is absent, causing emblem to search for the extension which
                     // defines the 'cmd' command
            .ext.cmd // qualifier is 'ext', causing emblem to specifically search the 'ext'
                     // extension for a definition of 'cmd'

            An empty qualifier (such as in `..cmd`) would represent an invalid extension name.

            If a literal dot is required to precede an unqualified command invocation, use glue:

            .~.cmd // represents a '.', glued to a call to 'cmd').
        "}
    }
}
