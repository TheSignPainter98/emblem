use crate::Log;
use emblem_core::{
    context::{Module as EmblemModule, ModuleVersion as EmblemModuleVersion},
    Version as EmblemVersion,
};
use serde::Deserialize as Deserialise;
use std::collections::HashMap;

pub(crate) fn load_str(src: &str) -> Result<DocManifest<'_>, Box<Log<'_>>> {
    // TODO(kcza): parse the errors into something pretty
    let parsed: DocManifest<'_> =
        serde_yaml::from_str(src).map_err(|e| Log::error(e.to_string()))?;

    parsed.validate().map_err(|e| Log::error(&*e))?;

    Ok(parsed)
}

#[derive(Debug, Deserialise)]
#[serde(deny_unknown_fields)]
pub(crate) struct DocManifest<'m> {
    pub name: &'m str,
    #[serde(rename = "emblem")]
    pub emblem_version: Version,
    pub authors: Option<Vec<&'m str>>,
    pub keywords: Option<Vec<&'m str>>,
    pub requires: Option<HashMap<&'m str, Module<'m>>>,
}

impl<'m> DocManifest<'m> {
    fn validate(&self) -> Result<(), String> {
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
pub(crate) struct Module<'m> {
    rename_as: Option<&'m str>,
    tag: Option<&'m str>,
    hash: Option<&'m str>,
    branch: Option<&'m str>,
    args: Option<HashMap<&'m str, &'m str>>,
}

impl<'m> Module<'m> {
    #[allow(unused)]
    pub fn rename_as(&self) -> Option<&'m str> {
        self.rename_as
    }

    #[allow(unused)]
    pub fn version(&self) -> ModuleVersion<'m> {
        if let Some(tag) = self.tag {
            return ModuleVersion::Tag(tag);
        }
        if let Some(branch) = self.branch {
            return ModuleVersion::Branch(branch);
        }
        if let Some(hash) = self.hash {
            return ModuleVersion::Hash(hash);
        }
        panic!("internal error: no version specified for {self:?}");
    }

    #[allow(unused)]
    pub fn args(&self) -> Option<&HashMap<&'m str, &'m str>> {
        match &self.args {
            None => None,
            Some(a) => Some(a),
        }
    }

    pub fn validate(&self, name: &str) -> Result<(), String> {
        match (&self.tag, &self.branch, &self.hash) {
            (Some(_), None, None) | (None, Some(_), None) | (None, None, Some(_)) => Ok(()),
            (None, None, None) => Err("expected `tag` or `hash` field".into()),
            _ => Err(format!("multiple version specifiers found for {name}")),
        }
    }
}

impl<'m> From<Module<'m>> for EmblemModule<'m> {
    fn from(module: Module<'m>) -> Self {
        Self::new(
            module.rename_as,
            module.version().into(),
            module.args.unwrap_or_default(),
        )
    }
}

#[derive(Debug, Eq, PartialEq)]
pub enum ModuleVersion<'m> {
    Tag(&'m str),
    Branch(&'m str),
    Hash(&'m str),
}

impl<'m> From<ModuleVersion<'m>> for EmblemModuleVersion<'m> {
    fn from(version: ModuleVersion<'m>) -> Self {
        match version {
            ModuleVersion::Tag(t) => Self::Tag(t),
            ModuleVersion::Branch(t) => Self::Branch(t),
            ModuleVersion::Hash(h) => Self::Hash(h),
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
            "#
            .into(),
        );
        let manifest = load_str(&raw).unwrap();

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
            "#
            .into(),
        );
        let manifest = load_str(&raw).unwrap();

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
        let missing_err = load_str(&missing).unwrap_err();
        let re = Regex::new("emblem: unknown variant `null`, expected").unwrap();
        let msg = missing_err.msg();
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
        let unknown_err = load_str(&unknown).unwrap_err();
        let re = Regex::new("emblem: unknown variant `UNKNOWN`, expected").unwrap();
        let msg = unknown_err.msg();
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
        let err = load_str(&raw).unwrap_err();
        let re = Regex::new("expected `tag` or `hash` field").unwrap();
        let msg = err.msg();
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
        let err = load_str(&raw).unwrap_err();
        let re = Regex::new("unknown field `INTERLOPER`").unwrap();
        let msg = err.msg();
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
            "#));
            let err = load_str(&raw).unwrap_err();
            let re = Regex::new("multiple version specifiers found for bar").unwrap();
            let msg = err.msg();
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
