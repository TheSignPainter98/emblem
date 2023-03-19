use crate::{ast::parsed::ParsedFile, build::typesetter::doc::Doc};

mod doc;

// TODO(kcza): typesettable file -> [fragment]

#[allow(unused)]
pub struct Typesetter {}

pub fn typeset(parsed_doc: ParsedFile<'_>) -> Result<(), ()> {
    let doc = Doc::from(parsed_doc);
    println!("{doc:#?}");
    Ok(())
}
