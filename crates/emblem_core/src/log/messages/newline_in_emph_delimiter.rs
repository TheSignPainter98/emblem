use crate::log::messages::Message;
use crate::log::{Log, Note, Src};
use crate::parser::Location;
use crate::FileContentSlice;
use derive_new::new;

#[derive(Default, new)]
pub struct NewlineInEmphDelimiter {
    delimiter_start_loc: Location,
    newline_loc: Location,
    expected: FileContentSlice,
}

impl Message for NewlineInEmphDelimiter {
    fn log(self) -> Log {
        Log::error(format!("newline in ‘{}’ emphasis", self.expected)).with_src(
            Src::new(&self.delimiter_start_loc.span_to(&self.newline_loc))
                .with_annotation(Note::error(&self.newline_loc, "newline found here"))
                .with_annotation(Note::info(
                    &self.delimiter_start_loc,
                    "in emphasis started here",
                )),
        )
    }
}
