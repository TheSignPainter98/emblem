use crate::log::messages::Message;
use crate::log::{Log, Note, Src};
use crate::parser::Location;
use derive_new::new;

#[derive(Default, new)]
pub struct NewlineInEmphDelimiter<'i> {
    delimiter_start_loc: Location<'i>,
    newline_loc: Location<'i>,
    expected: &'i str,
}

impl<'i> Message<'i> for NewlineInEmphDelimiter<'i> {
    fn log(self) -> Log<'i> {
        Log::error(format!("newline in ‘{}’ emphasis", self.expected)).src(
            Src::new(&self.delimiter_start_loc.span_to(&self.newline_loc))
                .annotate(Note::error(&self.newline_loc, "newline found here"))
                .annotate(Note::info(
                    &self.delimiter_start_loc,
                    "in emphasis started here",
                )),
        )
    }
}
