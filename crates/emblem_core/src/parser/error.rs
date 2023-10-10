use crate::parser::{
    lexer::{LexicalError, Tok},
    Point,
};
use lalrpop_util::ParseError as LalrpopParseError;

pub type ParseError = LalrpopParseError<Point, Tok, Box<LexicalError>>;
