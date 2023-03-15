use crate::log::messages::{Message, UnexpectedEOF};
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
            token: Tok::Newline { at_eof: false },
            expected: Default::default(),
        }
    }
}

impl<'i> Message<'i> for UnexpectedToken<'i> {
    fn log(self) -> Log<'i> {
        if matches!(self.token, Tok::Newline { at_eof: true }) {
            return UnexpectedEOF::new(self.loc.end(), vec![]).log();
        }
        Log::error(format!("unexpected {}", self.token))
            .with_src(Src::new(&self.loc).with_annotation(Note::error(&self.loc, "found here")))
            .with_expected(self.expected)
    }
}
