use crate::{Context, Log};
use serde::Deserialize as Deserialise;
use std::{collections::HashMap, fs};

pub(crate) fn load(ctx: &mut Context, manifest: String) -> Result<DocManifest<'_>, Log<'_>> {
    let content = match fs::read_to_string(&manifest) {
        Ok(c) => c,
        Err(e) => return Err(Log::error(e.to_string())),
    };
    let file = ctx.alloc_file(manifest, content);

    load_str(file.content())
}

pub(crate) fn load_str(src: &str) -> Result<DocManifest<'_>, Log<'_>> {
    // TODO(kcza): parse the errors into something pretty
    let parsed: DocManifest<'_> = serde_yaml::from_str(src).map_err(|e| Log::error(e.to_string()))?;

    parsed.validate().map_err(|e| Log::error(*e))?;

    Ok(parsed)
}

#[derive(Debug, Deserialise)]
#[serde(deny_unknown_fields)]
pub(crate) struct DocManifest<'m> {
    name: &'m str,
    authors: Option<Vec<&'m str>>,
    requires: Option<HashMap<&'m str, Module<'m>>>,
    output: Option<&'m str>,
}

impl<'m> DocManifest<'m> {
    #[allow(unused)]
    pub fn name(&self) -> &'m str {
        &self.name
    }

    #[allow(unused)]
    pub fn authors(&self) -> Option<&[&'m str]> {
        match &self.authors {
            None => None,
            Some(a) => Some(a.as_slice()),
        }
    }

    #[allow(unused)]
    pub fn requires(&self) -> Option<&HashMap<&'m str, Module<'m>>> {
        match &self.requires {
            None => None,
            Some(a) => Some(a),
        }
    }

    #[allow(unused)]
    pub fn output(&self) -> Option<&'m str> {
        self.output
    }

    fn validate(&self) -> Result<(), Box<String>> {
        if let Some(requires) = &self.requires {
            for spec in requires.values() {
                spec.validate()?;
            }
        }
        Ok(())
    }
}

#[derive(Debug, Deserialise, Eq, PartialEq)]
#[serde(rename_all = "kebab-case", deny_unknown_fields)]
pub(crate) struct Module<'m> {
    rename_as: Option<&'m str>,
    tag: Option<&'m str>,
    hash: Option<&'m str>,
    args: Option<HashMap<&'m str, &'m str>>,
}

impl<'m> Module<'m> {
    #[allow(unused)]
    pub fn rename_as(&self) -> Option<&'m str> {
        self.rename_as
    }

    #[allow(unused)]
    pub fn version(&self) -> Option<ModuleVersion<'m>> {
        if let Some(tag) = self.tag {
            return Some(ModuleVersion::Tag(tag));
        }
        if let Some(hash) = self.hash {
            return Some(ModuleVersion::Hash(hash));
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

    pub fn validate(&self) -> Result<(), Box<String>> {
        match (&self.tag, &self.hash) {
            (Some(_), None) | (None, Some(_)) => Ok(()),
            (None, None) => Err(Box::new("expected `tag` or `hash` field".into())),
            _ => Err(Box::new("multiple version specifiers found".into())),
        }
    }
}

#[derive(Debug, Eq, PartialEq)]
pub enum ModuleVersion<'m> {
    Tag(&'m str),
    Hash(&'m str),
}

#[cfg(test)]
mod test {
    use super::*;
    use regex::Regex;

    #[test]
    fn ok_minimal() {
        let raw = textwrap::dedent(
            r#"
            name: foo
        "#
            .into(),
        );
        let manifest = load_str(&raw).unwrap();

        assert_eq!("foo", manifest.name());
        assert_eq!(None, manifest.authors());
        assert_eq!(None, manifest.requires());
        assert_eq!(None, manifest.output());
    }

    #[test]
    fn ok_maximal() {
        let raw = textwrap::dedent(
            r#"
            name: foo
            authors:
            - Gordon
            - Eli
            - Isaac
            - Walter
            requires:
              foo-tagged:
                tag: edge
                rename-as: qux
                args:
                  key1: value1
                  key2: value2
              bar-hashed:
                hash: 0123456789abcdef
            output: pdf
        "#
            .into(),
        );
        let manifest = load_str(&raw).unwrap();

        assert_eq!("foo", manifest.name());
        assert_eq!(
            ["Gordon", "Eli", "Isaac", "Walter"],
            manifest.authors().unwrap()
        );

        {
            let requires = manifest.requires().unwrap();
            {
                let foo_tagged = requires.get("foo-tagged").unwrap();
                assert_eq!("qux", foo_tagged.rename_as().unwrap());
                assert_eq!(ModuleVersion::Tag("edge"), foo_tagged.version().unwrap());
                assert_eq!(&"value1", foo_tagged.args().unwrap().get("key1").unwrap());
                assert_eq!(&"value2", foo_tagged.args().unwrap().get("key2").unwrap());
            }

            {
                let baz_hashed = requires.get("bar-hashed").unwrap();
                assert_eq!(None, baz_hashed.rename_as());
                assert_eq!(
                    ModuleVersion::Hash("0123456789abcdef"),
                    baz_hashed.version().unwrap()
                );
                assert_eq!(None, baz_hashed.args());
            }
        }

        assert_eq!("pdf", manifest.output().unwrap());
    }

    #[test]
    fn missing_version() {
        let raw = textwrap::dedent(
            r#"
            name: foo
            requires:
              bar:
                args:
                  asdf: fdas
        "#
            .into(),
        );
        let err = load_str(&raw).unwrap_err();
        let re = Regex::new("expected `tag` or `hash` field").unwrap();
        let msg = err.msg();
        assert!(re.is_match(msg), "Unknown message doesn't match regex '{re:?}': got {msg}");
    }

    #[test]
    fn extra_fields() {
        let raw = textwrap::dedent(
            r#"
            INTERLOPER: true
            name: foo
        "#
            .into(),
        );
        let err = load_str(&raw).unwrap_err();
        let re = Regex::new("unknown field `INTERLOPER`").unwrap();
        let msg = err.msg();
        assert!(re.is_match(msg), "Unknown message doesn't match regex '{re:?}': got {msg}");
    }
}

// #[derive(Debug, Deserialise)]
// #[serde(rename_all = "kebab-case", deny_unknown_fields)]
// struct PackageManifest<'m> {
//     name: &'m str,
//     authors: Vec<&'m str>,
//     keywords: Option<Vec<&'m str>>,
//     requires: Option<HashMap<&'m str, PackageDependency<'m>>>,
//     provides_outputs: Vec<&'m str>,
//     args: PackageArgSpec<'m>,
// }

// #[derive(Debug, Deserialise)]
// #[serde(rename_all = "kebab-case", deny_unknown_fields)]
// struct PackageDependency<'m> {
//     branch: Option<&'m str>,
//     tag: Option<&'m str>,
//     commit: Option<&'m str>,
//     rename_as: Option<&'m str>,
// }

// #[derive(Debug, Deserialise)]
// #[serde(deny_unknown_fields, bound(deserialize = "'de: 'm"))]
// struct PackageArgSpec<'m> {
//     mandatory: Vec<&'m str>,
//     optional: Vec<&'m str>,
// }
