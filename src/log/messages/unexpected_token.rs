use crate::log::messages::Message;
use crate::log::{Log, Note, Src};
use crate::parser::{lexer::Tok, Location};
use derive_new::new;

#[derive(Debug, new)]
pub struct UnexpectedToken<'i> {
    loc: Location<'i>,
    token: Tok<'i>,
    expected: Vec<String>,
}

impl Default for UnexpectedToken<'_> {
    fn default() -> Self {
        Self {
            loc: Default::default(),
            token: Tok::Newline,
            expected: Default::default(),
        }
    }
}

impl<'i> Message<'i> for UnexpectedToken<'i> {
    fn log(self) -> Log<'i> {
        Log::error("unexpected token")
            .src(Src::new(&self.loc).annotate(Note::error(
                &self.loc,
                format!("found a {} here", self.token),
            )))
            .expect_one_of(&self.expected)
    }
}
