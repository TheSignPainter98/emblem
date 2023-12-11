use crate::{Context, Error, Result};
use arg_parser::InitCmd;
use camino::{Utf8Path, Utf8PathBuf};
use derive_new::new;
use emblem_core::log::Logger;
use git2::{Repository, RepositoryInitOptions};
use std::io::ErrorKind;
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
pub struct Initialiser<T: AsRef<Utf8Path>> {
    dir: T,
}

impl<T: AsRef<Utf8Path>> Initialiser<T> {
    pub fn run<L: Logger>(&self, _: &mut Context<L>) -> Result<()> {
        let p;
        let dir = {
            if !self.dir.as_ref().is_absolute() && !self.dir.as_ref().starts_with("./") {
                p = Utf8PathBuf::from(".").join(&self.dir);
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

impl From<&InitCmd> for Initialiser<Utf8PathBuf> {
    fn from(cmd: &InitCmd) -> Self {
        Self::new(Utf8PathBuf::from(cmd.dir.clone()))
    }
}

impl<T: AsRef<Utf8Path>> Initialiser<T> {
    /// Construct the contents of the manifest file
    fn generate_manifest(&self) -> Result<String> {
        let name = self.dir.as_ref().file_name().unwrap_or("emblem-document");

        Ok(indoc::formatdoc!(
            r#"
                [document]
                name = "{}"
                emblem = "1.0"

                # Use `em add <package>` to make <package> available to this document
            "#,
            name.replace('"', r#"\""#),
        ))
    }

    /// Try to create a new file with given contents. Optionally skip if file is already present.
    fn try_create_file(&self, path: &Utf8Path, contents: &str) -> Result<()> {
        match OpenOptions::new().write(true).create_new(true).open(path) {
            Ok(mut file) => {
                Ok(writeln!(file, "{}", contents.trim()).map_err(|e| Error::io(path, e))?)
            }
            Err(e) if e.kind() == ErrorKind::AlreadyExists => Ok(()),
            Err(e) => Err(Error::io(path, e)),
        }
    }

    /// Create a new code repository at the given path.
    fn init_repo(&self) -> Result<Repository> {
        Ok(Repository::init_opts(
            self.dir.as_ref(),
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

    fn do_init<L: Logger>(ctx: &mut Context<L>, dir: &Utf8Path) -> Result<()> {
        Initialiser::new(dir).run(ctx)
    }

    fn test_files(
        dir: &Utf8Path,
        expected_main_content: &str,
        expected_manifest_content: &str,
    ) -> Result<()> {
        let dot_git = dir.join(".git");
        assert!(dot_git.exists(), "no .git");
        assert!(dot_git.is_dir(), ".git is not a directory");

        let dot_gitignore = dir.join(".gitignore");
        assert!(dot_gitignore.exists(), "no .gitignore");
        assert!(dot_gitignore.is_file(), ".gitignore is not a file");

        const IGNORES: &[&str] = &["*.pdf"];

        let lines: Vec<String> =
            BufReader::new(File::open(&dot_gitignore).map_err(|e| Error::io(&dot_gitignore, e))?)
                .lines()
                .map(|r| r.map_err(|e| Error::io(&dot_gitignore, e)))
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
            let main_file = dir.join(main_file_name);
            assert!(main_file.exists(), "no main.em");
            assert!(main_file.is_file(), "main.em is not a file");
            let found_content =
                &fs::read_to_string(&main_file).map_err(|e| Error::io(&main_file, e))?;

            assert_eq!(expected_main_content, found_content);
            let ctx = Context::test_new();
            assert!(parser::parse(
                ctx.alloc_file_name(main_file.as_str()),
                ctx.alloc_file_content(expected_main_content)
            )
            .is_ok());
        }

        {
            let manifest_file_name = "emblem.yml";
            let manifest_file = dir.join(manifest_file_name);
            assert!(manifest_file.exists(), "no emblem.yml");
            assert!(manifest_file.is_file(), "main.em is not a file");
            let found_content =
                &fs::read_to_string(&manifest_file).map_err(|e| Error::io(&manifest_file, e))?;

            assert_eq!(expected_manifest_content, found_content);
            DocManifest::try_from(&found_content[..]).unwrap();
        }

        Ok(())
    }

    #[test]
    fn empty_dir() -> Result<()> {
        let tmpdir = tempfile::tempdir().unwrap();
        let tmpdir_path = Utf8PathBuf::try_from(tmpdir.path().to_owned()).unwrap();

        let mut ctx = Context::test_new();
        let result = do_init(&mut ctx, &tmpdir_path);
        assert!(result.is_ok(), "unexpected error: {}", result.unwrap_err());

        let expected_manifest_contents = &indoc::formatdoc!(
            r#"
                [document]
                name = "{}"
                emblem = "1.0"

                # Use `em add <package>` to make <package> available to this document
            "#,
            tmpdir_path.file_name().expect("tmpdir has no file name"),
        );
        test_files(
            &tmpdir_path,
            &MAIN_CONTENTS[1..],
            expected_manifest_contents,
        )
    }

    #[test]
    fn non_empty_dir() -> Result<()> {
        let tmpdir = tempfile::tempdir().unwrap();
        let tmpdir_path = Utf8PathBuf::try_from(tmpdir.path().to_owned()).unwrap();

        let main_file_path = tmpdir_path.join("main.em");
        let main_file_content = "hello, world!";
        fs::write(&main_file_path, main_file_content).map_err(|e| Error::io(&main_file_path, e))?;

        let manifest_file_path = tmpdir_path.join("emblem.yml");
        let manifest_file_content = indoc::indoc!(
            r#"
                [document]
                name = "asdf"
                emblem = "1.0"
            "#
        );
        fs::write(&manifest_file_path, manifest_file_content)
            .map_err(|e| Error::io(&manifest_file_path, e))?;

        {
            let mut ctx = Context::test_new();
            let result = do_init(&mut ctx, &tmpdir_path);
            assert!(result.is_ok(), "unexpected error: {}", result.unwrap_err());
            test_files(&tmpdir_path, main_file_content, manifest_file_content)?;
        }

        {
            let mut ctx = Context::test_new();
            let result = do_init(&mut ctx, &tmpdir_path);
            assert!(result.is_ok(), "unexpected error: {}", result.unwrap_err());
            test_files(&tmpdir_path, main_file_content, manifest_file_content)
        }
    }

    #[test]
    fn init_repo() -> Result<()> {
        let tmpdir = tempfile::tempdir().unwrap();
        let tmpdir_path = Utf8PathBuf::try_from(tmpdir.path().to_owned()).unwrap();

        let initialiser = Initialiser::new(&tmpdir_path);
        initialiser.init_repo()?;

        let dot_git = tmpdir_path.join(".git");
        assert!(dot_git.exists(), "no .git");
        assert!(dot_git.is_dir(), ".git is not a directory");

        initialiser.init_repo()?;

        Ok(())
    }
}
