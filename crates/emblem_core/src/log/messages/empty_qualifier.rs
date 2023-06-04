use crate::log::messages::Message;
use crate::log::{Log, Note, Src};
use crate::parser::Location;
use derive_new::new;

#[derive(Default, new)]
pub struct EmptyQualifier<'i> {
    loc: Location<'i>,
    qualifier_loc: Location<'i>,
}

impl<'i> Message<'i> for EmptyQualifier<'i> {
    fn id() -> &'static str {
        "E004"
    }

    fn log(self) -> Log<'i> {
        Log::error("empty qualifier in command name")
            .with_id(Self::id())
            .explainable()
            .with_src(
                Src::new(&self.loc).with_annotation(Note::error(&self.qualifier_loc, "found here")),
            )
    }

    fn explain(&self) -> &'static str {
        concat!(
            "This error means that a command has been called with an empty qualifier. ",
            "This likely originates from a call which looks like `..cmd`. ",
            "There are two ways to call a command:\n",
            "\n",
            ".cmd     // qualifier is absent, causing emblem to search for the extension which defines the 'cmd' command\n",
            ".ext.cmd // qualifier is 'ext', causing emblem to specifically search the 'ext' extension for a definition of 'cmd'\n",
            "\n",
            "An empty qualifier (such as in `..cmd`) would represent an invalid extension name.\n",
            "\n",
            "If a literal dot is required to precede an unqualified command invocation, use glue:\n",
            "\n",
            ".~.cmd // represents a '.', glued to a call to 'cmd').",
        )
    }
}
