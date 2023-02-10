use crate::log::messages::Message;
use crate::log::{Log, Note, Src};
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
            .explainable()
            .src(
                Src::new(&self.arg_start_loc.span_to(&self.newline_loc))
                    .annotate(Note::error(&self.newline_loc, "newline found here"))
                    .annotate(Note::info(
                        &self.arg_start_loc,
                        "in inline argument started here",
                    )),
            )
            .help("consider using trailer (colon) arguments")
    }

    fn explain(&self) -> &'static str {
        concat!(
            "This error means that a newline was detected early in the parsing of arguments. ",
            "Command arguments have two forms:\n",
            "\n",
            "```\n",
            ".command{inline-arg-1}{inline-arg-2}{...}: remainder-arg\n",
            "// or\n",
            ".command{inline-arg-1}{inline-arg-2}{...}:\n",
            "\ttrailer\n",
            "\targ\n",
            "\t1\n",
            "::\n",
            "\ttrailer\n",
            "\targ\n",
            "\t2\n",
            "::\n",
            "\t...\n",
            "```\n",
            "\n",
            "If you are an extension author, consider ordering arguments so your users are encouraged to place longer ones later.",
        )
    }
}
