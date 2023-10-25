use derive_new::new;

use crate::log::messages::Message;
use crate::log::{Note, Src};
use crate::parser::Location;
use crate::Log;

#[derive(Default, new)]
pub struct UnclosedVerbatim {
    loc: Location,
}

impl Message for UnclosedVerbatim {
    fn log(self) -> Log {
        Log::error(format!("unclosed verbatim block"))
            .with_src(Src::new(&self.loc).with_annotation(Note::error(&self.loc, "found here")))
            .with_note(format!("expected corresponding {}", self.loc))
    }
}
