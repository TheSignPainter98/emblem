use crate::parser::Location;

#[derive(Debug)]
pub struct Node<'input> {
    name: Text<'input>,
    location: Location<'input>,
}

#[derive(Debug)]
enum Text<'input> {
    Owned(String),
    Borrowed(&'input str),
}

// impl AsRef
// impl Into<String>
