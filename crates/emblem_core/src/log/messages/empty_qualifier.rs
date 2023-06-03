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
            "This error means that a command-call has been made where the qualifier cannot refer to any package.\n",
            "\n",
            ".cmd     // causes emblem to search for a single extension which defines a 'cmd' command\n",
            ".pkg.cmd // causes emblem to specifically search the 'pkg' extension for a definition of 'cmd'\n",
            "\n",
            "If multiple extensions define 'foo', then `.foo` would be ambiguous. ",
            "In this case, a qualifier may be added to tell emblem to look at the definitions made by a particular extension (i.e. `.pkg.cmd` is never ambiguous).\n",
            "\n",
            "As extension names have at least one character, an empty qualifier cannot be valid; two dots cannot start a command invocation (e.g. `..cmd`). ",
            "If two dots are required, consider adding glue to separate the tokens (e.g. `.~.cmd` is a dot followed immediately by a call to 'cmd')."
        )
    }
}
