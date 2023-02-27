use crate::repo;
use crate::Action;
use crate::Context;
use crate::Log;
use derive_new::new;
use std::error::Error;
use std::path::{Path, PathBuf};
use std::{
    fs::OpenOptions,
    io::{self, Write},
};

static MAIN_CONTENTS: &str = r#"
# Welcome! Welcome to Emblem.

You have chosen, or been chosen, to relocate to one of our finest remaining typesetters.
"#;

static GITIGNORE_CONTENTS: &str = r#"
# Output files
*.pdf
"#;

#[derive(new)]
pub struct Initialiser {
    dir: String,
    allow_non_empty_dir: bool,
}

impl Action for Initialiser {
    fn run<'em>(&self, _: &'em mut Context) -> Vec<Log<'em>> {
        match self.run_internal() {
            Ok(_) => vec![],
            Err(e) => vec![Log::error(e.to_string())],
        }
    }
}

impl Initialiser {
    fn run_internal(&self) -> Result<(), Box<dyn Error>> {
        let dir = {
            let dir = Path::new(&self.dir);
            if !dir.is_absolute() && !dir.starts_with("./") {
                PathBuf::from(".").join(dir)
            } else {
                dir.into()
            }
        };

        if !self.allow_non_empty_dir
            && dir.try_exists().ok() == Some(true)
            && dir.read_dir()?.next().is_some()
        {
            // TODO(kcza): change error kind to DirectoryNotEmpty once stable
            return Err(Box::new(io::Error::new(
                io::ErrorKind::Other,
                format!("Directory {:?} is not empty", dir),
            )));
        }
        let git_ignore = dir.join(".gitignore");
        let main_file = dir.join("main.em");

        repo::init(&dir)?;

        self.try_create_file(&git_ignore, GITIGNORE_CONTENTS)?;
        self.try_create_file(&main_file, MAIN_CONTENTS)?;

        eprintln!("New emblem document created in {:?}", main_file);

        Ok(())
    }

    /// Try to create a new file with given contents. Optionally skip if file is already present.
    fn try_create_file(&self, path: &Path, contents: &str) -> Result<(), io::Error> {
        match OpenOptions::new().write(true).create_new(true).open(path) {
            Ok(mut file) => write!(file, "{}", contents.trim()),
            Err(e) => {
                if self.allow_non_empty_dir {
                    Ok(())
                } else {
                    Err(e)
                }
            }
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::parser;
    use std::{
        fs::{self, File},
        io::{BufRead, BufReader},
    };
    use tempfile::TempDir;

    fn do_init<'em>(ctx: &'em mut Context, tmpdir: &TempDir, dir_not_empty: bool) -> Vec<Log<'em>> {
        Initialiser::new(tmpdir.path().to_str().unwrap().to_owned(), dir_not_empty).run(ctx)
    }

    fn test_files(dir: &TempDir, expected_ast_structure: &str) -> Result<(), Box<dyn Error>> {
        let dot_git = dir.path().join(".git");
        assert!(dot_git.exists(), "no .git");
        assert!(dot_git.is_dir(), ".git is not a directory");

        let dot_gitignore = dir.path().join(".gitignore");
        assert!(dot_gitignore.exists(), "no .gitignore");
        assert!(dot_gitignore.is_file(), ".gitignore is not a file");

        const IGNORES: &[&str] = &["*.pdf"];

        let lines: Vec<String> = BufReader::new(File::open(dot_gitignore)?)
            .lines()
            .filter_map(|l| l.ok())
            .collect::<Vec<_>>();

        for ignore in IGNORES {
            assert!(
                lines.contains(&ignore.to_string()),
                "Missing ignore: {}",
                ignore
            );
        }

        let main_file = dir.path().join("main.em");
        assert!(main_file.exists(), "no main.em");
        assert!(main_file.is_file(), "main.em is not a file");

        parser::test::assert_structure(
            "main.em",
            &fs::read_to_string(main_file)?,
            expected_ast_structure,
        );

        Ok(())
    }

    #[test]
    fn empty_dir() -> Result<(), Box<dyn Error>> {
        let tmpdir = tempfile::tempdir()?;

        let mut ctx = Context::new();
        let problems = do_init(&mut ctx, &tmpdir, false);
        assert!(problems.is_empty(), "unexpected problems: {:?}", problems);

        test_files(&tmpdir, "File[Par[[$h1{[Word(Welcome!)|< >|Word(Welcome)|< >|Word(to)|< >|Word(Emblem.)]}]]|Par[[Word(You)|< >|Word(have)|< >|Word(chosen,)|< >|Word(or)|< >|Word(been)|< >|Word(chosen,)|< >|Word(to)|< >|Word(relocate)|< >|Word(to)|< >|Word(one)|< >|Word(of)|< >|Word(our)|< >|Word(finest)|< >|Word(remaining)|< >|Word(typesetters.)]]]")
    }

    #[test]
    fn non_empty_dir() -> Result<(), Box<dyn Error>> {
        let tmpdir = tempfile::tempdir()?;
        let main_file_path = tmpdir.path().join("main.em");
        let main_file_content = "hello, world!";
        fs::write(&main_file_path, main_file_content)?;

        {
            let mut ctx = Context::new();
            let problems = do_init(&mut ctx, &tmpdir, false);
            assert!(!problems.is_empty(), "expected problems");
        }

        {
            let mut ctx = Context::new();
            let problems = do_init(&mut ctx, &tmpdir, true);
            assert!(problems.is_empty(), "unexpected problems: {:?}", problems);
        }

        test_files(&tmpdir, "File[Par[[Word(hello,)|< >|Word(world!)]]]")
    }
}
