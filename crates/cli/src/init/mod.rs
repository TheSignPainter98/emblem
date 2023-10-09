use crate::{Context, Result};
use arg_parser::InitCmd;
use derive_new::new;
use git2::{Repository, RepositoryInitOptions};
use std::io::ErrorKind;
use std::path::{Path, PathBuf};
use std::{fs::OpenOptions, io::Write};

static MAIN_CONTENTS: &str = r#"
# Welcome! Welcome to Emblem.

You have chosen, or been chosen, to relocate to one of our finest remaining typesetters.
"#;

static GITIGNORE_CONTENTS: &str = r#"
# Output files
*.pdf
"#;

#[derive(new)]
pub struct Initialiser<T: AsRef<Path>> {
    dir: T,
}

impl<T: AsRef<Path>> Initialiser<T> {
    pub fn run(&self, _: &mut Context) -> Result<()> {
        let p;
        let dir = {
            if !self.dir.as_ref().is_absolute() && !self.dir.as_ref().starts_with("./") {
                p = PathBuf::from(".").join(&self.dir);
                &p
            } else {
                self.dir.as_ref()
            }
        };

        let git_ignore = dir.join(".gitignore");
        let main_file = dir.join("main.em");
        let manifest_file = dir.join("emblem.yml");

        self.init_repo()?;

        self.try_create_file(&git_ignore, GITIGNORE_CONTENTS)?;
        self.try_create_file(&main_file, MAIN_CONTENTS)?;
        self.try_create_file(&manifest_file, &self.generate_manifest()?)?;

        Ok(())
    }
}

impl From<&InitCmd> for Initialiser<PathBuf> {
    fn from(cmd: &InitCmd) -> Self {
        Self::new(PathBuf::from(cmd.dir.clone()))
    }
}

impl<T: AsRef<Path>> Initialiser<T> {
    /// Construct the contents of the manifest file
    fn generate_manifest(&self) -> Result<String> {
        let name = self
            .dir
            .as_ref()
            .file_name()
            .map(|s| {
                s.to_str()
                    .expect("directory name contains non-unicode characters")
            })
            .unwrap_or("emblem-document");

        let manifest = format!(
            r#"
                name: {name}
                authors: []
                keywords: []
                emblem: v1.0

                # Use `em add <package>` to make <package> available to this document
                requires: {{}}
            "#
        )
        .replace("                ", "");
        Ok(manifest)
    }

    /// Try to create a new file with given contents. Optionally skip if file is already present.
    fn try_create_file(&self, path: &Path, contents: &str) -> Result<()> {
        match OpenOptions::new().write(true).create_new(true).open(path) {
            Ok(mut file) => Ok(writeln!(file, "{}", contents.trim())?),
            Err(e) if e.kind() == ErrorKind::AlreadyExists => Ok(()),
            Err(e) => Err(e.into()),
        }
    }

    /// Create a new code repository at the given path.
    fn init_repo(&self) -> Result<Repository> {
        Ok(Repository::init_opts(
            &self.dir,
            RepositoryInitOptions::new().mkdir(true),
        )?)
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::Result;
    use crate::{manifest::DocManifest, Error};
    use emblem_core::parser;
    use std::{
        fs::{self, File},
        io::{BufRead, BufReader},
    };
    use tempfile::TempDir;

    fn do_init(ctx: &mut Context, tmpdir: &TempDir) -> Result<()> {
        Initialiser::new(tmpdir).run(ctx)
    }

    fn test_files(
        dir: &TempDir,
        expected_main_content: &str,
        expected_manifest_content: &str,
    ) -> Result<()> {
        let dot_git = dir.path().join(".git");
        assert!(dot_git.exists(), "no .git");
        assert!(dot_git.is_dir(), ".git is not a directory");

        let dot_gitignore = dir.path().join(".gitignore");
        assert!(dot_gitignore.exists(), "no .gitignore");
        assert!(dot_gitignore.is_file(), ".gitignore is not a file");

        const IGNORES: &[&str] = &["*.pdf"];

        let lines: Vec<String> = BufReader::new(File::open(dot_gitignore)?)
            .lines()
            .map(|r| r.map_err(Error::from))
            .collect::<Result<Vec<_>>>()?;

        for ignore in IGNORES {
            assert!(
                lines.contains(&ignore.to_string()),
                "Missing ignore: {}",
                ignore
            );
        }

        {
            let main_file_name = "main.em";
            let main_file = dir.path().join(main_file_name);
            assert!(main_file.exists(), "no main.em");
            assert!(main_file.is_file(), "main.em is not a file");
            let found_content = &fs::read_to_string(&main_file)?;

            assert_eq!(expected_main_content, found_content);
            let ctx = Context::new();
            assert!(parser::parse(
                ctx.alloc_file_name(main_file.as_path().to_str().unwrap()),
                ctx.alloc_file_content(expected_main_content)
            )
            .is_ok());
        }

        {
            let manifest_file_name = "emblem.yml";
            let manifest_file = dir.path().join(manifest_file_name);
            assert!(manifest_file.exists(), "no main.em");
            assert!(manifest_file.is_file(), "main.em is not a file");
            let found_content = &fs::read_to_string(&manifest_file)?;

            assert_eq!(expected_manifest_content, found_content);
            DocManifest::try_from(&found_content[..]).unwrap();
        }

        Ok(())
    }

    #[test]
    fn empty_dir() -> Result<()> {
        let tmpdir = tempfile::tempdir()?;

        let mut ctx = Context::new();
        let result = do_init(&mut ctx, &tmpdir);
        assert!(result.is_ok(), "unexpected error: {}", result.unwrap_err());

        let expected_manifest_contents = textwrap::dedent(
            &format!(
                r#"
                    name: {}
                    authors: []
                    keywords: []
                    emblem: v1.0

                    # Use `em add <package>` to make <package> available to this document
                    requires: {{}}
                "#,
                tmpdir
                    .path()
                    .file_name()
                    .expect("tmpdir has no file name")
                    .to_str()
                    .expect("tmpdir contained non-ascii characters"),
            )[1..],
        );
        test_files(&tmpdir, &MAIN_CONTENTS[1..], &expected_manifest_contents)
    }

    #[test]
    fn non_empty_dir() -> Result<()> {
        let tmpdir = tempfile::tempdir()?;

        let main_file_path = tmpdir.path().join("main.em");
        let main_file_content = "hello, world!";
        fs::write(main_file_path, main_file_content)?;

        let manifest_file_path = tmpdir.path().join("emblem.yml");
        let manifest_file_content = "name: asdf\nemblem: v1.0";
        fs::write(manifest_file_path, manifest_file_content)?;

        {
            let mut ctx = Context::new();
            let result = do_init(&mut ctx, &tmpdir);
            assert!(result.is_ok(), "unexpected error: {}", result.unwrap_err());
            test_files(&tmpdir, main_file_content, manifest_file_content)?;
        }

        {
            let mut ctx = Context::new();
            let result = do_init(&mut ctx, &tmpdir);
            assert!(result.is_ok(), "unexpected error: {}", result.unwrap_err());
            test_files(&tmpdir, main_file_content, manifest_file_content)
        }
    }

    #[test]
    fn init_repo() -> Result<()> {
        let dir = tempfile::tempdir()?;

        let initialiser = Initialiser::new(dir.path());
        initialiser.init_repo()?;

        let dot_git = dir.path().join(".git");
        assert!(dot_git.exists(), "no .git");
        assert!(dot_git.is_dir(), ".git is not a directory");

        initialiser.init_repo()?;

        Ok(())
    }
}
