use crate::log::messages::Message;
use crate::log::{Log, Msg, Src};
use crate::parser::{lexer::Tok, Location};

pub struct UnexpectedToken<'i> {
    loc: Location<'i>,
    token: Tok<'i>,
    expected: Vec<String>,
}

impl<'i> UnexpectedToken<'i> {
    pub fn new(loc: Location<'i>, token: Tok<'i>, expected: Vec<String>) -> Self {
        Self {
            loc,
            token,
            expected,
        }
    }
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
    fn id() -> &'static str
    where
        Self: Sized,
    {
        "E003"
    }

    fn log(self) -> Log<'i> {
        Log::error("unexpected token")
            .id(Self::id())
            .src(Src::new(&self.loc).annotate(Msg::error(
                &self.loc,
                format!("found a {} here", self.token),
            )))
            .expect_one_of(&self.expected)
    }
}
