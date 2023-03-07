use crate::args::TypesetterArgs;
use crate::ast::parsed::ParsedFile;

mod doc;

// TODO(kcza): parsed file -> typesettable file
// TODO(kcza): typesettable file -> [fragment]

pub struct Typesetter {
    pub fn with(args: TypesetterArgs, style: StyleArgs, extensions: ExtensionArgs) -> Self {
    }
}

pub fn typeset<'ctx>(args: TypesetterArgs, doc: ParsedFile<'ctx>) -> Result<(), ()> {
    println!("{args:?}, {doc:?}");
    Ok(())
}
