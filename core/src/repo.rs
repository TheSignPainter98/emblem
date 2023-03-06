#[cfg(feature = "git2")]
use git2::{ErrorCode, Repository, Status, StatusOptions};
use std::error::Error;
use std::path::Path;

#[cfg(not(feature = "git2"))]
#[allow(dead_code)]
pub fn is_dirty(_dir: &Path) -> Result<bool, Box<dyn Error>> {
    panic!(
        "internal error: emblem_core::repo::is_dirty cannot be called without the 'git2' feature"
    );
}

/// Check whether there is a dirty repository at the given path.
#[cfg(feature = "git2")]
#[allow(dead_code)]
pub fn is_dirty(dir: &Path) -> Result<bool, Box<dyn Error>> {
    let repo = match Repository::open(dir) {
        Ok(r) => r,
        Err(e) if e.code() == ErrorCode::NotFound => return Ok(false),
        Err(e) => return Err(Box::new(e)),
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
            return Ok(true);
        }
    }

    Ok(false)
}

#[cfg(test)]
mod test {
    use super::*;
    use std::{fs::File, io::Write};

    use git2::{Repository, RepositoryInitOptions};

    #[cfg(feature = "git2")]
    #[test]
    fn dirt_detection() -> Result<(), Box<dyn Error>> {
        let dir = tempfile::tempdir()?;

        assert!(!is_dirty(dir.path())?);

        Repository::init_opts(dir.path(), RepositoryInitOptions::new().mkdir(true))?;

        assert!(!is_dirty(dir.path())?);

        {
            let mut file = File::create(dir.path().join("dirt.txt"))?;
            file.write_all(b"some dirt")?;
        }

        assert!(is_dirty(dir.path())?);

        Ok(())
    }
}
