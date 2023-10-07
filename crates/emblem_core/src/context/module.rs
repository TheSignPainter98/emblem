use std::collections::HashMap;

#[derive(Debug, Eq, PartialEq)]
pub struct Module {
    name: String,
    source: String,
    rename_as: Option<String>,
    version: ModuleVersion,
    args: HashMap<String, String>,
}

impl Module {
    pub fn new(
        source: String,
        rename_as: Option<String>,
        version: ModuleVersion,
        args: HashMap<String, String>,
    ) -> Self {
        let name = source
            .rfind('/')
            .map(|i| &source[1 + i..])
            .unwrap_or(&source)
            .to_owned();
        Self {
            name,
            source,
            rename_as,
            version,
            args,
        }
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn source(&self) -> &str {
        &self.source
    }

    pub fn rename_as(&self) -> Option<&str> {
        self.rename_as.as_deref()
    }

    pub fn version(&self) -> &ModuleVersion {
        &self.version
    }

    pub fn args(&self) -> &HashMap<String, String> {
        &self.args
    }

    pub fn args_mut(&mut self) -> &mut HashMap<String, String> {
        &mut self.args
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ModuleVersion {
    Tag(String),
    Branch(String),
    Hash(String),
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn getters() {
        let source = "github.com/TheSignPainter98/some-repo";
        let rename = "some-new-name";
        let version = ModuleVersion::Tag("some-tag".into());
        let args: HashMap<String, String> = [("foo", "bar"), ("baz", "qux")]
            .into_iter()
            .map(|(k, v)| (k.into(), v.into()))
            .collect();

        let dep = Module::new(
            source.to_owned(),
            Some(rename.to_owned()),
            version.clone(),
            args.clone(),
        );
        assert_eq!("some-repo", dep.name());
        assert_eq!(source, dep.source());
        assert_eq!(rename, dep.rename_as().unwrap());
        assert_eq!(&version, dep.version());
        assert_eq!(&args, dep.args());
    }

    #[test]
    fn rename_as() {
        assert_eq!(
            None,
            Module::new(
                ".".into(),
                None,
                ModuleVersion::Tag("bar".into()),
                HashMap::new()
            )
            .rename_as()
        );

        let expected = "new-name".to_string();
        assert_eq!(
            expected.clone(),
            Module::new(
                ".".into(),
                Some(expected),
                ModuleVersion::Tag("bar".into()),
                HashMap::new()
            )
            .rename_as()
            .unwrap()
        );
    }

    #[test]
    fn version() {
        let tag = ModuleVersion::Tag("bar".into());
        assert_eq!(
            &tag.clone(),
            Module::new(".".into(), None, tag, HashMap::new()).version()
        );

        let branch = ModuleVersion::Branch("bar".into());
        assert_eq!(
            &branch.clone(),
            Module::new(".".into(), None, branch, HashMap::new()).version()
        );

        let hash = ModuleVersion::Hash("bar".into());
        assert_eq!(
            &hash.clone(),
            Module::new(".".into(), None, hash, HashMap::new()).version()
        );
    }
}
