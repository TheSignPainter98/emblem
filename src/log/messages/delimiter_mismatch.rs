use crate::log::messages::Message;
use crate::log::{Log, Note, Src};
use crate::parser::Location;
use derive_new::new;

#[derive(Default, new)]
pub struct DelimiterMismatch<'i> {
    loc: Location<'i>,
    to_close_loc: Location<'i>,
    expected: &'i str,
}

impl<'i> Message<'i> for DelimiterMismatch<'i> {
    fn id() -> &'static str {
        "E003"
    }

    fn log(self) -> Log<'i> {
        Log::error("mismatching delimiter")
            .id(Self::id())
            .explainable()
            .src(
                Src::new(&self.to_close_loc.span_to(&self.loc))
                    .annotate(Note::error(&self.loc, format!("expected ‘{}’ here", self.expected)))
                    .annotate(Note::info(&self.to_close_loc, format!("to close ‘{}’ found here", self.expected)))
            )
    }

    fn explain(&self) -> &'static str {
        concat!(
            "This error means that a closing delimiter was found which did not match the most recently opened one. ",
            "This may be the fault of a typo, but in some cases this may be caused by emblem incorrectly parsing different delimiters which use the same character, which can cause some local ambiguity about how to handle some tokens.\n",
            "\n",
            "For example:\n",
            "___foo bar_ baz__ should be parsed as __(_foo bar_) baz__, but\n",
            "___foo bar__ baz_ should be parsed as _(__(foo bar)__ baz_, \n",
            "however, when Emblem sees the `___`, it does not know how it should break it, which may result in this error if the wrong choice has been made.\n",
            "\n",
            "This problem can be entirely avoided by sticking to the convention that _italics use underscores_ and **bold use asterisks.**",
        )
    }
}