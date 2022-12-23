use super::text::Text;

#[derive(Debug)]
pub enum Content<'i> {
    Call {
        name: Text<'i>,
        args: Vec<Content<'i>>,
    },
    Word(Text<'i>),
    Whitespace(&'i str),
    Comment(&'i str),
    MultiLineComment(MultiLineComment<'i>),
}

#[derive(Debug)]
pub enum MultiLineComment<'i> {
    Word(&'i str),
    Whitespace(&'i str),
    Indented(Box<MultiLineComment<'i>>),
    Nested(Box<MultiLineComment<'i>>),
}
