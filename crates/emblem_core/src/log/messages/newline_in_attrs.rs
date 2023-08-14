use crate::log::messages::Message;
use crate::log::{Log, Note, Src};
use crate::parser::Location;
use derive_new::new;

#[derive(Default, new)]
pub struct NewlineInAttrs {
    attr_start_loc: Location,
    newline_loc: Location,
}

impl<'i> Message<'i> for NewlineInAttrs {
    fn log(self) -> Log {
        Log::error("newline in attributes").with_src(
            Src::new(&self.attr_start_loc.span_to(&self.newline_loc))
                .with_annotation(Note::error(&self.newline_loc, "newline found here"))
                .with_annotation(Note::info(
                    &self.attr_start_loc,
                    "in inline attributes started here",
                )),
        )
    }
}
