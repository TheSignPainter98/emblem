use crate::log::messages::Message;
use crate::log::{Log, Note, Src};
use crate::parser::Location;
use derive_new::new;

#[derive(Default, new)]
pub struct UnexpectedChar {
    loc: Location,
    found: char,
}

impl<'i> Message<'i> for UnexpectedChar {
    fn log(self) -> Log {
        Log::error(format!("unexpected character ‘{}’", self.found))
            .with_src(Src::new(&self.loc).with_annotation(Note::error(&self.loc, "found here")))
    }
}
