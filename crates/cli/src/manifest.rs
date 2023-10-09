use crate::{Error, Result};
use emblem_core::{
    context::{Module as EmblemModule, ModuleVersion as EmblemModuleVersion},
    Version as EmblemVersion,
};
use serde::Deserialize as Deserialise;
use std::collections::HashMap;

#[derive(Debug, Deserialise)]
#[serde(deny_unknown_fields)]
pub(crate) struct DocManifest {
    pub name: String,
    #[serde(rename = "emblem")]
    pub emblem_version: Version,
    pub authors: Option<Vec<String>>,
    pub keywords: Option<Vec<String>>,
    pub requires: Option<HashMap<String, Module>>,
}

impl TryFrom<&str> for DocManifest {
    type Error = Error;

    fn try_from(src: &str) -> Result<Self> {
        let parsed: DocManifest = serde_yaml::from_str(src)?;
        parsed.validate()?;
        Ok(parsed)
    }
}

impl DocManifest {
    fn validate(&self) -> Result<()> {
        if let Some(requires) = &self.requires {
            for (name, ext) in requires {
                ext.validate(name)?;
            }
        }
        Ok(())
    }
}

#[derive(Clone, Copy, Debug, Deserialise, Eq, PartialEq)]
pub(crate) enum Version {
    #[serde(rename = "v1.0")]
    V1_0,
}

impl Version {
    #[allow(unused)]
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::V1_0 => "v1.0",
        }
    }
}

impl From<Version> for EmblemVersion {
    fn from(version: Version) -> Self {
        match version {
            Version::V1_0 => Self::V1_0,
        }
    }
}

#[derive(Debug, Deserialise, Eq, PartialEq)]
#[serde(rename_all = "kebab-case", deny_unknown_fields)]
pub(crate) struct Module {
    rename_as: Option<String>,
    tag: Option<String>,
    hash: Option<String>,
    branch: Option<String>,
    args: Option<HashMap<String, String>>,
}

impl Module {
    #[allow(unused)]
    pub fn rename_as(&self) -> Option<&str> {
        self.rename_as.as_deref()
    }

    #[allow(unused)]
    pub fn version(&self) -> ModuleVersion<'_> {
        if let Some(tag) = &self.tag {
            return ModuleVersion::Tag(tag);
        }
        if let Some(branch) = &self.branch {
            return ModuleVersion::Branch(branch);
        }
        if let Some(hash) = &self.hash {
            return ModuleVersion::Hash(hash);
        }
        panic!("internal error: no version specified for {self:?}");
    }

    #[allow(unused)]
    pub fn args(&self) -> Option<&HashMap<String, String>> {
        match &self.args {
            None => None,
            Some(a) => Some(a),
        }
    }

    pub fn validate(&self, name: &str) -> Result<()> {
        match (&self.tag, &self.branch, &self.hash) {
            (Some(_), None, None) | (None, Some(_), None) | (None, None, Some(_)) => Ok(()),
            (None, None, None) => Err(Error::manifest_invalid("expected `tag` or `hash` field")),
            _ => Err(Error::manifest_invalid(format!(
                "multiple version specifiers found for {name}"
            ))),
        }
    }

    pub fn into_module(self, source: String) -> EmblemModule {
        let version = self.version().into();
        EmblemModule::new(
            source,
            self.rename_as,
            version,
            self.args.unwrap_or_default(),
        )
    }
}

#[derive(Debug, Eq, PartialEq)]
pub enum ModuleVersion<'m> {
    Tag(&'m str),
    Branch(&'m str),
    Hash(&'m str),
}

impl From<ModuleVersion<'_>> for EmblemModuleVersion {
    fn from(version: ModuleVersion) -> Self {
        match version {
            ModuleVersion::Tag(t) => Self::Tag(t.to_string()),
            ModuleVersion::Branch(t) => Self::Branch(t.to_string()),
            ModuleVersion::Hash(h) => Self::Hash(h.to_string()),
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use itertools::Itertools;
    use regex::Regex;

    #[test]
    fn ok_minimal() {
        let raw = textwrap::dedent(
            r#"
                name: foo
                emblem: v1.0
            "#,
        );
        let manifest = DocManifest::try_from(&raw[..]).unwrap();

        assert_eq!("foo", manifest.name);
        assert_eq!(Version::V1_0, manifest.emblem_version);
        assert_eq!(None, manifest.authors);
        assert_eq!(None, manifest.requires);
    }

    #[test]
    fn ok_maximal() {
        let raw = textwrap::dedent(
            r#"
                name: foo
                emblem: v1.0
                authors:
                - Gordon
                - Eli
                - Isaac
                - Walter
                keywords:
                - DARGH!
                - NO!
                - STAHP!
                - HUEAG!
                requires:
                  foo-tagged:
                    tag: edge
                    rename-as: qux
                    args:
                      key1: value1
                      key2: value2
                  bar-branched:
                    branch: dev
                  baz-hashed:
                    hash: 0123456789abcdef
            "#,
        );
        let manifest = DocManifest::try_from(&raw[..]).unwrap();

        assert_eq!("foo", manifest.name);
        assert_eq!(
            &["Gordon", "Eli", "Isaac", "Walter"],
            manifest.authors.unwrap().as_slice()
        );
        assert_eq!(
            &["DARGH!", "NO!", "STAHP!", "HUEAG!"],
            manifest.keywords.unwrap().as_slice()
        );
        assert_eq!(Version::V1_0, manifest.emblem_version);

        {
            let requires = manifest.requires.unwrap();
            {
                let foo_tagged = requires.get("foo-tagged").unwrap();
                assert_eq!("qux", foo_tagged.rename_as().unwrap());
                assert_eq!(ModuleVersion::Tag("edge"), foo_tagged.version());
                assert_eq!(&"value1", foo_tagged.args().unwrap().get("key1").unwrap());
                assert_eq!(&"value2", foo_tagged.args().unwrap().get("key2").unwrap());
            }

            {
                let bar_branched = requires.get("bar-branched").unwrap();
                assert_eq!(None, bar_branched.rename_as());
                assert_eq!(ModuleVersion::Branch("dev"), bar_branched.version());
                assert_eq!(None, bar_branched.args());
            }

            {
                let baz_hashed = requires.get("baz-hashed").unwrap();
                assert_eq!(
                    ModuleVersion::Hash("0123456789abcdef"),
                    baz_hashed.version()
                );
            }
        }
    }

    #[test]
    fn incorrect_emblem_version() {
        let missing = textwrap::dedent(
            r#"
                name: foo
                emblem: null
            "#,
        );
        let missing_err = DocManifest::try_from(&missing[..]).unwrap_err();
        let re = Regex::new("emblem: unknown variant `null`, expected").unwrap();
        let msg = &missing_err.to_string();
        assert!(
            re.is_match(msg),
            "Unknown message doesn't match regex '{re:?}': got {msg}"
        );

        let unknown = textwrap::dedent(
            r#"
                name: foo
                emblem: UNKNOWN
            "#,
        );
        let unknown_err = DocManifest::try_from(&unknown[..]).unwrap_err();
        let re = Regex::new("emblem: unknown variant `UNKNOWN`, expected").unwrap();
        let msg = &unknown_err.to_string();
        assert!(
            re.is_match(msg),
            "Unknown message doesn't match regex '{re:?}': got {msg}"
        );
    }

    #[test]
    fn missing_dependency_version() {
        let raw = textwrap::dedent(
            r#"
                name: foo
                emblem: v1.0
                requires:
                  bar:
                    args:
                      asdf: fdas
            "#,
        );
        let err = DocManifest::try_from(&raw[..]).unwrap_err();
        let re = Regex::new("expected `tag` or `hash` field").unwrap();
        let msg = &err.to_string();
        assert!(
            re.is_match(msg),
            "Unknown message doesn't match regex '{re:?}': got {msg}"
        );
    }

    #[test]
    fn extra_fields() {
        let raw = textwrap::dedent(
            r#"
                INTERLOPER: true
                name: foo
            "#,
        );
        let err = DocManifest::try_from(&raw[..]).unwrap_err();
        let re = Regex::new("unknown field `INTERLOPER`").unwrap();
        let msg = &err.to_string();
        assert!(
            re.is_match(msg),
            "Unknown message doesn't match regex '{re:?}': got {msg}"
        );
    }

    #[test]
    fn multiple_version_specifiers() {
        let specifiers = ["tag: asdf", "branch: asdf", "hash: asdf"];
        for (specifier_1, specifier_2) in specifiers
            .iter()
            .cartesian_product(specifiers.iter())
            .filter(|(s, t)| s != t)
        {
            let raw = textwrap::dedent(&format!(
                r#"
                    name: foo
                    emblem: v1.0
                    requires:
                      bar:
                        {specifier_1}
                        {specifier_2}
                "#
            ));
            let err = DocManifest::try_from(&raw[..]).unwrap_err();
            let re = Regex::new("multiple version specifiers found for bar").unwrap();
            let msg = &err.to_string();
            assert!(
                re.is_match(msg),
                "Unknown message doesn't match regex '{re:?}': got {msg}"
            );
        }
    }
}

// #[derive(Debug, Deserialise)]
// #[serde(rename_all = "kebab-case", deny_unknown_fields)]
// struct PackageManifest<'m> {
//     name: &'m str,
//     authors: Vec<&'m str>,
//     keywords: Option<Vec<&'m str>>,
//     requires: Option<HashMap<ModuleName<'m>, ModuleDependency<'m>>>,
//     provides_outputs: Vec<&'m str>,
//     args: PackageArgSpec<'m>,
// }

// #[derive(Debug, Deserialise)]
// #[serde(rename_all = "kebab-case", deny_unknown_fields)]
// struct ModuleDependency<'m> {
//     branch: Option<&'m str>,
//     tag: Option<&'m str>,
//     commit: Option<&'m str>,
//     rename_as: Option<&'m str>,
// }

// #[derive(Debug, Deserialise)]
// #[serde(deny_unknown_fields, bound(deserialize = "'de: 'm"))]
// struct ModuleArgSpec<'m> {
//     mandatory: Vec<&'m str>,
//     optional: Vec<&'m str>,
// }
