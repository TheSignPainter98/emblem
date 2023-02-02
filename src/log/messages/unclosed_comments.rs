use crate::log::messages::Message;
use crate::log::{Log, Msg, Src};
use crate::parser::Location;
use derive_new::new;

#[derive(Default, new)]
pub struct UnclosedComments<'i> {
    unclosed: Vec<Location<'i>>,
}

impl<'i> Message<'i> for UnclosedComments<'i> {
    fn log(self) -> Log<'i> {
        let msg = if self.unclosed.len() > 1 {
            "unclosed comments"
        } else {
            "unclosed comment"
        };

        let mut ret = Log::error(msg);
        for loc in self.unclosed {
            ret = ret.src(Src::new(&loc).annotate(Msg::error(&loc, "found here")))
        }
        ret
    }
}
