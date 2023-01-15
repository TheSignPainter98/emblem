use crate::args::InitCmd;
use git2::{Repository, RepositoryInitOptions};
use std::error::Error;
use std::path::{Path, PathBuf};
use std::{
    fs::OpenOptions,
    io::{self, Write},
};

static MAIN_CONTENTS: &str = r#"
# Emblem document

Welcome to _Emblem._
"#;

static GITIGNORE_CONTENTS: &str = r#"
# Output files
*.pdf
"#;

/// Initialise a new Emblem project
pub fn init(cmd: InitCmd) -> Result<(), Box<dyn Error>> {
    let dir = {
        let dir = Path::new(cmd.dir());
        if !dir.is_absolute() && !dir.starts_with("./") {
            PathBuf::from(".").join(dir)
        } else {
            dir.into()
        }
    };

    if !cmd.dir_not_empty()
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

    Repository::init_opts(dir, RepositoryInitOptions::new().mkdir(true))?;

    try_create_file(&git_ignore, GITIGNORE_CONTENTS, cmd.dir_not_empty())?;
    try_create_file(&main_file, MAIN_CONTENTS, cmd.dir_not_empty())?;

    eprintln!("New emblem document created in {:?}", main_file);

    Ok(())
}

/// Try to create a new file with given contents. Optionally skip if file is already present.
fn try_create_file(path: &Path, contents: &str, dir_not_empty: bool) -> Result<(), io::Error> {
    match OpenOptions::new().write(true).create_new(true).open(path) {
        Ok(mut file) => write!(file, "{}", contents.trim()),
        Err(e) => {
            if dir_not_empty {
                Ok(())
            } else {
                Err(e)
            }
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use tempfile::TempDir;

    fn do_init(tmpdir: &TempDir, dir_not_empty: bool) -> Result<(), Box<dyn Error>> {
        let cmd = InitCmd::new(tmpdir.path().to_str().unwrap().to_owned(), dir_not_empty);
        init(cmd)?;
        Ok(())
    }

    mod empty_dir {
        use super::*;
        use std::{
            fs::{self, File},
            io::{BufRead, BufReader},
        };

        fn test_files(dir: &TempDir) -> Result<(), Box<dyn Error>> {
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
            // TODO(kcza): test the main file builds

            Ok(())
        }

        #[test]
        fn empty_dir() -> Result<(), Box<dyn Error>> {
            let tmpdir = tempfile::tempdir()?;
            do_init(&tmpdir, false)?;
            test_files(&tmpdir)
        }

        #[test]
        fn non_empty_dir() -> Result<(), Box<dyn Error>> {
            let tmpdir = tempfile::tempdir()?;
            let main_file_path = tmpdir.path().join("main.em");
            let main_file_content = "hello, world!";
            fs::write(&main_file_path, main_file_content)?;

            assert!(do_init(&tmpdir, false).is_err());
            assert!(
                do_init(&tmpdir, true).is_ok(),
                "failed to force file initialisation"
            );

            test_files(&tmpdir)?;

            assert_eq!(main_file_content, fs::read_to_string(&main_file_path)?);

            Ok(())
        }
    }
}
