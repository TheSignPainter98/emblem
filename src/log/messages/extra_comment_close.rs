use crate::log::messages::Message;
use crate::log::{Log, Msg, Src};
use crate::parser::Location;
use derive_new::new;

#[derive(Default, new)]
pub struct ExtraCommentClose<'i> {
    loc: Location<'i>,
}

impl<'i> Message<'i> for ExtraCommentClose<'i> {
    fn log(self) -> Log<'i> {
        Log::error("no comment to close")
            .src(Src::new(&self.loc).annotate(Msg::error(&self.loc, "found here")))
    }
}
