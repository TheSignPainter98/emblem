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
    #[serde(rename = "document")]
    pub(crate) metadata: DocMetadata,
    #[serde(rename = "requires")]
    pub(crate) dependencies: Option<HashMap<String, Module>>,
}

impl TryFrom<&str> for DocManifest {
    type Error = Error;

    fn try_from(src: &str) -> Result<Self> {
        let parsed: DocManifest = toml_edit::de::from_str(src)?;
        parsed.validate()?;
        Ok(parsed)
    }
}

impl DocManifest {
    fn validate(&self) -> Result<()> {
        if let Some(dependencies) = &self.dependencies {
            for (name, ext) in dependencies {
                ext.validate(name)?;
            }
        }
        Ok(())
    }
}

#[derive(Debug, Deserialise)]
#[serde(deny_unknown_fields)]
pub(crate) struct DocMetadata {
    pub(crate) name: String,
    #[serde(rename = "emblem")]
    pub(crate) emblem_version: Version,
    pub(crate) authors: Option<Vec<String>>,
    pub(crate) keywords: Option<Vec<String>>,
}

#[derive(Clone, Copy, Debug, Deserialise, Eq, PartialEq)]
pub(crate) enum Version {
    #[serde(rename = "1.0")]
    V1_0,
}

impl Version {
    #[allow(unused)]
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::V1_0 => "1.0",
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
        let raw = indoc::indoc!(
            r#"
                [document]
                name = "foo"
                emblem = "1.0"
            "#,
        );
        let manifest = DocManifest::try_from(&raw[..]).unwrap();

        assert_eq!("foo", manifest.metadata.name);
        assert_eq!(Version::V1_0, manifest.metadata.emblem_version);
        assert_eq!(None, manifest.metadata.authors);
        assert_eq!(None, manifest.dependencies);
    }

    #[test]
    fn ok_maximal() {
        let raw = indoc::indoc!(
            r#"
                [document]
                name = "foo"
                emblem = "1.0"
                authors = [
                    "Gordon",
                    "Eli",
                    "Isaac",
                    "Walter",
                ]
                keywords = [
                    "DARGH!",
                    "NO!",
                    "STAHP!",
                    "HUEAG!",
                ]

                [requires.foo-tagged]
                tag = "edge"
                rename-as = "qux"
                args = { key1 = "value1", key2 = "value2" }

                [requires.bar-branched]
                branch = "dev"

                [requires.baz-hashed]
                hash = "0123456789abcdef"
            "#,
        );
        let manifest = DocManifest::try_from(&raw[..]).unwrap();

        assert_eq!("foo", manifest.metadata.name);
        assert_eq!(
            &["Gordon", "Eli", "Isaac", "Walter"],
            manifest.metadata.authors.unwrap().as_slice()
        );
        assert_eq!(
            &["DARGH!", "NO!", "STAHP!", "HUEAG!"],
            manifest.metadata.keywords.unwrap().as_slice()
        );
        assert_eq!(Version::V1_0, manifest.metadata.emblem_version);

        {
            let dependencies = manifest.dependencies.unwrap();
            {
                let foo_tagged = dependencies.get("foo-tagged").unwrap();
                assert_eq!("qux", foo_tagged.rename_as().unwrap());
                assert_eq!(ModuleVersion::Tag("edge"), foo_tagged.version());
                assert_eq!(&"value1", foo_tagged.args().unwrap().get("key1").unwrap());
                assert_eq!(&"value2", foo_tagged.args().unwrap().get("key2").unwrap());
            }

            {
                let bar_branched = dependencies.get("bar-branched").unwrap();
                assert_eq!(None, bar_branched.rename_as());
                assert_eq!(ModuleVersion::Branch("dev"), bar_branched.version());
                assert_eq!(None, bar_branched.args());
            }

            {
                let baz_hashed = dependencies.get("baz-hashed").unwrap();
                assert_eq!(
                    ModuleVersion::Hash("0123456789abcdef"),
                    baz_hashed.version()
                );
            }
        }
    }

    #[test]
    fn incorrect_emblem_version() {
        let missing = indoc::indoc!(
            r#"
                [document]
                name = "foo"
            "#,
        );
        let missing_err = DocManifest::try_from(&missing[..]).unwrap_err();
        let re = Regex::new("missing field `emblem`").unwrap();
        let msg = &missing_err.to_string();
        assert!(
            re.is_match(msg),
            "Unknown message doesn't match regex '{re:?}': got {msg}"
        );

        let unknown = indoc::indoc!(
            r#"
                [document]
                name = "foo"
                emblem = "UNKNOWN"
            "#,
        );
        let unknown_err = DocManifest::try_from(&unknown[..]).unwrap_err();
        let re = Regex::new("unknown variant `UNKNOWN`, expected").unwrap();
        let msg = &unknown_err.to_string();
        assert!(
            re.is_match(msg),
            "Unknown message doesn't match regex '{re:?}': got {msg}"
        );
    }

    #[test]
    fn missing_dependency_version() {
        let raw = indoc::indoc!(
            r#"
                [document]
                name = "foo"
                emblem = "1.0"

                [requires.bar]
                args = { asdf = "fdas" }
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
    fn extra_top_level_table() {
        let raw = indoc::indoc!(
            r#"
                [document]
                name = "foo"
                emblem = "1.0"

                [INTERLOPER]
            "#
        );
        let err = DocManifest::try_from(&raw[..]).unwrap_err();
        let re = Regex::new("unknown field `INTERLOPER`").unwrap();
        let msg = &err.to_string();
        assert!(
            re.is_match(msg),
            "Unknown messages doesn't match regex: '{re:?}': got {msg}"
        );
    }

    #[test]
    fn extra_fields() {
        let extra_metadata = indoc::indoc!(
            r#"
                [document]
                INTERLOPER = true
                name = "foo"
                emblem = "1.0"
            "#,
        );
        let err = DocManifest::try_from(&extra_metadata[..]).unwrap_err();
        let re = Regex::new("unknown field `INTERLOPER`").unwrap();
        let msg = &err.to_string();
        assert!(
            re.is_match(msg),
            "Unknown message doesn't match regex '{re:?}': got {msg}"
        );

        let extra_dependency_field = indoc::indoc!(
            r#"
                [document]
                name = "foo"
                emblem = "1.0"

                [requires.foo]
                tag = "1.0"
                INTERLOPER = true
            "#,
        );
        let err = DocManifest::try_from(&extra_dependency_field[..]).unwrap_err();
        let re = Regex::new("unknown field `INTERLOPER`").unwrap();
        let msg = &err.to_string();
        assert!(
            re.is_match(msg),
            "Unknown message doesn't match regex '{re:?}': got {msg}"
        );
    }

    #[test]
    fn multiple_version_specifiers() {
        let specifiers = [r#"tag = "asdf""#, r#"branch = "asdf""#, r#"hash = "asdf""#];
        for (specifier_1, specifier_2) in specifiers
            .iter()
            .cartesian_product(specifiers.iter())
            .filter(|(s, t)| s != t)
        {
            let raw = indoc::formatdoc!(
                r#"
                    [document]
                    name = "foo"
                    emblem = "1.0"

                    [requires.bar]
                    {specifier_1}
                    {specifier_2}
                "#
            );
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
