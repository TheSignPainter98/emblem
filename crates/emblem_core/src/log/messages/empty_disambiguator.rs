use crate::log::messages::Message;
use crate::log::{Log, Note, Src};
use crate::parser::Location;
use derive_new::new;

#[derive(Default, new)]
pub struct EmptyDisambiguator<'i> {
    loc: Location<'i>,
    disambiguator_loc: Location<'i>,
}

impl<'i> Message<'i> for EmptyDisambiguator<'i> {
    fn id() -> &'static str {
        "E004"
    }

    fn log(self) -> Log<'i> {
        Log::error("empty disambiguator in command name")
            .with_id(Self::id())
            .explainable()
            .with_src(
                Src::new(&self.loc)
                    .with_annotation(Note::error(&self.disambiguator_loc, "found here")),
            )
    }

    fn explain(&self) -> &'static str {
        concat!(
            "This error means that a command-call has been made where the disambiguator cannot refer to any package.\n",
            "\n",
            ".cmd     // causes emblem to search for a single extension which defines a 'cmd' command\n",
            ".pkg.cmd // causes emblem to specifically search the 'pkg' extension for a definition of 'cmd'\n",
            "\n",
            "If multiple extensions define 'foo', then `.foo` would be ambiguous. ",
            "In this case, a disambiguator may be added to tell emblem to look at the definitions made by a particular extension (i.e. `.pkg.cmd` is never ambiguous).\n",
            "\n",
            "As extension names have at least one character, an empty disambiguator cannot be valid; two dots cannot start a command invocation (e.g. `..cmd`). ",
            "If two dots are required, consider adding glue to separate the tokens (e.g. `.~.cmd` is a dot followed immediately by a call to 'cmd')."
        )
    }
}
