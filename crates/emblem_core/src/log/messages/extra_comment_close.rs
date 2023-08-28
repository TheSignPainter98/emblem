use crate::log::messages::Message;
use crate::log::{Log, Note, Src};
use crate::parser::Location;
use derive_new::new;

#[derive(Default, new)]
pub struct ExtraCommentClose {
    loc: Location,
}

impl Message for ExtraCommentClose {
    fn log(self) -> Log {
        Log::error("no comment to close")
            .with_src(Src::new(&self.loc).with_annotation(Note::error(&self.loc, "found here")))
    }
}
