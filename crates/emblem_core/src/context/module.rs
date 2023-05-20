use std::collections::HashMap;

use derive_new::new;

#[derive(new, Debug, Eq, PartialEq)]
pub struct Module<'m> {
    name: &'m str,
    source: &'m str,
    rename_as: Option<&'m str>,
    version: ModuleVersion<'m>,
    args: HashMap<&'m str, &'m str>,
}

impl<'m> Module<'m> {
    pub fn name_from_source(source: &'m str) -> &'m str {
        source
            .rfind('/')
            .map(|i| &source[1 + i..])
            .unwrap_or(source)
    }

    pub fn name(&self) -> &'m str {
        &self.name
    }

    pub fn source(&self) -> &'m str {
        &self.source
    }

    pub fn rename_as(&self) -> &Option<&'m str> {
        &self.rename_as
    }

    pub fn version(&self) -> ModuleVersion<'m> {
        self.version
    }

    pub fn args(&self) -> &HashMap<&'m str, &'m str> {
        &self.args
    }

    pub fn args_mut(&mut self) -> &mut HashMap<&'m str, &'m str> {
        &mut self.args
    }
}

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum ModuleVersion<'m> {
    Tag(&'m str),
    Branch(&'m str),
    Hash(&'m str),
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn getters() {
        let name = "some-name";
        let source = "github.com/TheSignPainter98/some-repo";
        let rename = "some-new-name";
        let version = ModuleVersion::Tag("some-tag");
        let args: HashMap<_, _> = [("foo", "bar"), ("baz", "qux")].into_iter().collect();

        let dep = Module::new(name, source, Some(rename), version, args.clone());
        assert_eq!(name, dep.name());
        assert_eq!(source, dep.source());
        assert_eq!(rename, dep.rename_as().unwrap());
        assert_eq!(version, dep.version());
        assert_eq!(&args, dep.args());
    }

    #[test]
    fn rename_as() {
        assert_eq!(
            &None,
            Module::new("foo", ".", None, ModuleVersion::Tag("bar"), HashMap::new()).rename_as()
        );

        let expected = "new-name";
        assert_eq!(
            expected,
            Module::new(
                "foo",
                ".",
                Some(expected),
                ModuleVersion::Tag("bar"),
                HashMap::new()
            )
            .rename_as()
            .unwrap()
        );
    }

    #[test]
    fn version() {
        let tag = ModuleVersion::Tag("bar");
        assert_eq!(tag, Module::new("foo", ".", None, tag, HashMap::new()).version());

        let branch = ModuleVersion::Branch("bar");
        assert_eq!(
            branch,
            Module::new("foo", ".", None, branch, HashMap::new()).version()
        );

        let hash = ModuleVersion::Hash("bar");
        assert_eq!(
            hash,
            Module::new("foo", ".", None, hash, HashMap::new()).version()
        );
    }
}
