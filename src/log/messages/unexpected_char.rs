use crate::log::messages::Message;
use crate::log::{Log, Note, Src};
use crate::parser::Location;
use derive_new::new;

#[derive(Default, new)]
pub struct UnexpectedChar<'i> {
    loc: Location<'i>,
    found: char,
}

impl<'i> Message<'i> for UnexpectedChar<'i> {
    fn log(self) -> Log<'i> {
        Log::error(format!("unexpected character '{:?}'", self.found))
            .src(Src::new(&self.loc).annotate(Note::error(&self.loc, "found here")))
    }
}
