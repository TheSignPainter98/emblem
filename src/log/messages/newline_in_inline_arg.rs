use crate::log::messages::Message;
use crate::log::{Log, Msg, Src};
use crate::parser::Location;
use derive_new::new;

#[derive(Default, new)]
pub struct NewlineInInlineArg<'i> {
    arg_start_loc: Location<'i>,
    newline_loc: Location<'i>,
}

impl<'i> Message<'i> for NewlineInInlineArg<'i> {
    fn id() -> &'static str {
        "E002"
    }

    fn log(self) -> Log<'i> {
        Log::error("newline in inline (curly-braced) arguments")
            .id(Self::id())
            .src(
                Src::new(&self.arg_start_loc.span_to(&self.newline_loc))
                    .annotate(Msg::error(&self.newline_loc, "newline found here"))
                    .annotate(Msg::info(
                        &self.arg_start_loc,
                        "in inline argument started here",
                    )),
            )
            .help("consider using trailer (colon) arguments")
    }
}
