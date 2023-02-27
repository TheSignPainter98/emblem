use crate::ast::parsed::ParsedFile;

pub struct Doc<'i> {
    name: &'i str,
    // root: Node<'i>,
}

impl<'i> From<ParsedFile<'i>> for Doc<'i> {
    fn from(parsed: ParsedFile<'i>) -> Self {
        Self { name: "foo" }
    }
}
