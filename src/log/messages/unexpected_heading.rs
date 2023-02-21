use crate::log::messages::Message;
use crate::log::{Log, Note, Src};
use crate::parser::Location;
use derive_new::new;

#[derive(Default, new)]
pub struct UnexpectedHeading<'i> {
    loc: Location<'i>,
}

impl<'i> Message<'i> for UnexpectedHeading<'i> {
    fn id() -> &'static str {
        "E004"
    }

    fn log(self) -> Log<'i> {
        Log::error("unexpected heading")
            .id(Self::id())
            .explainable()
            .src(Src::new(&self.loc).annotate(Note::error(&self.loc, "found here")))
            .help("headings should only appear at the start of lines")
    }
}
