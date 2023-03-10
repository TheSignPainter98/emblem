use crate::log::messages::Message;
use crate::log::{Log, Note, Src};
use crate::parser::Location;
use derive_new::new;

#[derive(Default, new)]
pub struct UnexpectedHeading<'i> {
    loc: Location<'i>,
}

impl<'i> Message<'i> for UnexpectedHeading<'i> {
    fn log(self) -> Log<'i> {
        Log::error("unexpected heading")
            .with_src(Src::new(&self.loc).with_annotation(Note::error(&self.loc, "found here")))
            .with_help("headings should only appear at the start of lines")
    }
}
