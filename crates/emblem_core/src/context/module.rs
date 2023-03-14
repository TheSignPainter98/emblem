use std::collections::HashMap;

use derive_new::new;

#[derive(Clone, Debug, Hash, Eq, PartialEq)]
pub struct ModuleName<'m> {
    name: &'m str,
    source: &'m str,
}

impl<'m> ModuleName<'m> {
    pub fn name(&self) -> &'m str {
        self.name
    }

    pub fn source(&self) -> &'m str {
        self.source
    }
}

impl<'m> From<&'m str> for ModuleName<'m> {
    fn from(source: &'m str) -> Self {
        Self {
            name: match source.find('/') {
                Some(idx) => &source[1 + idx..],
                None => source,
            },
            source,
        }
    }
}

#[derive(new, Debug, Eq, PartialEq)]
pub struct Module<'m> {
    rename_as: Option<&'m str>,
    version: ModuleVersion<'m>,
    args: HashMap<&'m str, &'m str>,
}

impl<'m> Module<'m> {
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
    Hash(&'m str),
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn getters() {
        let rename = "some-new-name";
        let version = ModuleVersion::Tag("some-tag");
        let args: HashMap<_, _> = [("foo", "bar"), ("baz", "qux")].into_iter().collect();

        let dep = Module::new(Some(rename), version, args.clone());
        assert_eq!(rename, dep.rename_as().unwrap());
        assert_eq!(version, dep.version());
        assert_eq!(&args, dep.args());
    }

    #[test]
    fn rename_as() {
        assert_eq!(
            &None,
            Module::new(None, ModuleVersion::Tag("foo"), HashMap::new()).rename_as()
        );

        let expected = "new-name";
        assert_eq!(
            expected,
            Module::new(
                Some(expected),
                ModuleVersion::Tag("foo"),
                HashMap::new()
            )
            .rename_as()
            .unwrap()
        );
    }

    #[test]
    fn version() {
        let tag = ModuleVersion::Tag("foo");
        assert_eq!(tag, Module::new(None, tag, HashMap::new()).version());

        let tag = ModuleVersion::Hash("bar");
        assert_eq!(tag, Module::new(None, tag, HashMap::new()).version());
    }
}
