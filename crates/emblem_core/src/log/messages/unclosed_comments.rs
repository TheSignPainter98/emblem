use crate::log::messages::Message;
use crate::log::{Log, Note, Src};
use crate::parser::Location;
use derive_new::new;

#[derive(Default, new)]
pub struct UnclosedComments {
    unclosed: Vec<Location>,
}

impl Message for UnclosedComments {
    fn log(self) -> Log {
        let msg = if self.unclosed.len() > 1 {
            "unclosed comments"
        } else {
            "unclosed comment"
        };

        let mut ret = Log::error(msg);
        for loc in self.unclosed {
            ret = ret.with_src(Src::new(&loc).with_annotation(Note::error(&loc, "found here")))
        }
        ret
    }
}
