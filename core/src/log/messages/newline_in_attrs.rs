use crate::log::messages::Message;
use crate::log::{Log, Note, Src};
use crate::parser::Location;
use derive_new::new;

#[derive(Default, new)]
pub struct NewlineInAttrs<'i> {
    attr_start_loc: Location<'i>,
    newline_loc: Location<'i>,
}

impl<'i> Message<'i> for NewlineInAttrs<'i> {
    fn log(self) -> Log<'i> {
        Log::error("newline in attributes").src(
            Src::new(&self.attr_start_loc.span_to(&self.newline_loc))
                .annotate(Note::error(&self.newline_loc, "newline found here"))
                .annotate(Note::info(
                    &self.attr_start_loc,
                    "in inline attributes started here",
                )),
        )
    }
}
