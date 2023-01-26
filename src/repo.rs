use git2::{Error, ErrorCode, Repository, RepositoryInitOptions, Status, StatusOptions};
use std::path::Path;

pub fn init(dir: &Path) -> Result<Repository, Error> {
    Repository::init_opts(dir, RepositoryInitOptions::new().mkdir(true))
}

#[allow(dead_code)]
pub fn is_clean(dir: &Path) -> Result<bool, Error> {
    let repo = match Repository::open(dir) {
        Ok(r) => r,
        Err(e) if e.code() == ErrorCode::NotFound => return Ok(true),
        Err(e) => return Err(e),
    };

    let mut opts = StatusOptions::new();
    opts.include_untracked(true);

    for status in repo.statuses(Some(&mut opts))?.iter() {
        let dirty_flags = Status::INDEX_NEW
            | Status::INDEX_RENAMED
            | Status::INDEX_TYPECHANGE
            | Status::INDEX_DELETED
            | Status::INDEX_MODIFIED
            | Status::WT_NEW
            | Status::WT_RENAMED
            | Status::WT_TYPECHANGE
            | Status::WT_DELETED
            | Status::WT_MODIFIED
            | Status::IGNORED
            | Status::CONFLICTED;
        if status.status().bits() & dirty_flags.bits() != 0 {
            return Ok(false);
        }
    }

    Ok(true)
}

#[cfg(test)]
mod test {
    use std::{error::Error, fs::File, io::Write};

    #[test]
    fn init() -> Result<(), Box<dyn Error>> {
        let dir = tempfile::tempdir()?;
        super::init(dir.path())?;

        let dot_git = dir.path().join(".git");
        assert!(dot_git.exists(), "no .git");
        assert!(dot_git.is_dir(), ".git is not a directory");

        Ok(())
    }

    #[test]
    fn is_clean() -> Result<(), Box<dyn Error>> {
        let dir = tempfile::tempdir()?;

        assert!(super::is_clean(dir.path())?);

        super::init(dir.path())?;

        println!("a");
        assert!(super::is_clean(dir.path())?);
        println!("b");

        {
            let mut file = File::create(dir.path().join("dirt.txt"))?;
            file.write(b"some dirt")?;
        }

        println!("c");
        assert!(!super::is_clean(dir.path())?);
        println!("d");

        Ok(())
    }
}
