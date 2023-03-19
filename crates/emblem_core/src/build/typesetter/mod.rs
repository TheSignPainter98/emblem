use crate::{ast::parsed::ParsedFile, build::typesetter::doc::Doc};

mod doc;

// TODO(kcza): typesettable file -> [fragment]

#[allow(unused)]
pub struct Typesetter {
}

pub fn typeset<'ctx>(parsed_doc: ParsedFile<'ctx>) -> Result<(), ()> {
    let doc = Doc::from(parsed_doc);
    println!("{doc:#?}");
    Ok(())
}
