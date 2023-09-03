use crate::log::messages::Message;
use crate::log::{Log, LogId, Note, Src};
use crate::parser::Location;
use derive_new::new;
use indoc::indoc;

#[derive(Default, new)]
pub struct NewlineInInlineArg {
    arg_start_loc: Location,
    newline_loc: Location,
}

impl Message for NewlineInInlineArg {
    fn id() -> LogId {
        "E002".into()
    }

    fn log(self) -> Log {
        Log::error("newline in inline (curly-braced) arguments")
            .with_id(Self::id())
            .explainable()
            .with_src(
                Src::new(&self.arg_start_loc.span_to(&self.newline_loc))
                    .with_annotation(Note::error(&self.newline_loc, "newline found here"))
                    .with_annotation(Note::info(
                        &self.arg_start_loc,
                        "in inline argument started here",
                    )),
            )
            .with_help("consider using trailer (colon) arguments")
    }

    fn explain(&self) -> &'static str {
        indoc! {"
            This error means that a newline was detected early in the parsing of arguments.
            Command arguments have two forms:

            .command{inline-arg-1}{inline-arg-2}{...}: remainder-arg
            // or
            .command{inline-arg-1}{inline-arg-2}{...}:
                trailer
                arg
                1
            ::
                trailer
                arg
                2
            ::
                ...

            If you are a module author, consider ordering arguments so your users are encouraged to
            place longer ones later
        "}
    }
}
