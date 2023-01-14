use crate::args::InitCmd;
use git2::{Repository, RepositoryInitOptions};
use std::error::Error;
use std::path::{Path, PathBuf};
use std::{fs, io};

static GITIGNORE_CONTENTS: &str = r#"
# Output files
*.pdf
"#;

pub fn init(cmd: InitCmd) -> Result<(), Box<dyn Error>> {
    let path = {
        let path = cmd.input.file.path();
        if path == None {
            return Err(Box::new(io::Error::new(
                io::ErrorKind::InvalidInput,
                format!("Cannot create new emblem project in {}", cmd.input.file),
            )));
        }
        let mut path = path.unwrap().join("main");
        if !path.is_absolute() {
            path = PathBuf::from(".").join(path);
        }
        path
    };

    let dir = path.parent().unwrap();

    if dir.try_exists().ok() == Some(true) && dir.read_dir()?.next().is_some() {
        // TODO(kcza): change error kind to DirectoryNotEmpty once stable
        return Err(Box::new(io::Error::new(
            io::ErrorKind::Other,
            format!("Directory {:?} is not empty", dir),
        )));
    }
    let git_repo = path.with_file_name(".git/");
    let git_ignore = path.with_file_name(".gitignore");
    let main_file = path.with_extension("em");

    eprintln!("Writing git repo: {:?}", git_repo);
    Repository::init_opts(
        git_repo.parent().unwrap(),
        RepositoryInitOptions::new()
            .description(cmd.title().unwrap_or("emblem file"))
            .mkdir(true)
            .no_reinit(true),
    )?;
    eprintln!("Writing file: {:?}", git_ignore);
    write_file(&git_ignore, GITIGNORE_CONTENTS)?;

    let main_contents = format!(
        r#"
# {}

Welcome to _Emblem._
"#,
        cmd.title().unwrap_or("Emblem document".into())
    );
    write_file(&main_file, &main_contents)?;

    eprintln!("New emblem document created in {:?}", main_file);

    Ok(())
}

fn write_file(file: &Path, contents: &str) -> Result<(), io::Error> {
    fs::write(file, &contents[1..contents.len() - 1])
}

#[cfg(test)]
mod test {
    use super::*;

    // TODO(kcza): test git initialisation
    // TODO(kcza): test what happens if a git repo is already present, or any of the files
    // are!
    // TODO(kcza): test stdin blocked
}
