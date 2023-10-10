use crate::{args::ArgPath, Error, Result};
use std::{
    fs,
    io::{self, Read},
    path,
};

#[cfg(test)]
use std::{
    fs::File,
    io::{BufReader, Stdin},
};

// #[derive(Clone, Debug, Default, PartialEq, Eq)]
// pub struct SearchPath {
//     path: Vec<path::PathBuf>,
// }

// impl SearchPath {
//     #[allow(dead_code)]
//     pub fn open<S, T>(&self, src: S, target: T) -> Result<SearchResult, io::Error>
//     where
//         S: Into<path::PathBuf>,
//         T: AsRef<path::Path>,
//     {
//         let target = target.as_ref();

//         if target.is_absolute() {
//             return Err(io::Error::new(
//                 io::ErrorKind::InvalidInput,
//                 format!("Absolute paths are forbidden: got {:?}", target,),
//             ));
//         }

//         let src = src.into().canonicalize()?;

//         let path = path::PathBuf::from(&src).join(target);
//         if path.starts_with(&src) {
//             if let Ok(file) = fs::File::open(&path) {
//                 if let Ok(metadata) = file.metadata() {
//                     if metadata.is_file() {
//                         let file = InputFile::from(file);
//                         return Ok(SearchResult { path, file });
//                     }
//                 }
//             }
//         }

//         for dir in self.normalised().path {
//             let path = {
//                 let p = path::PathBuf::from(&dir).join(target);
//                 match p.canonicalize() {
//                     Ok(p) => p,
//                     _ => continue,
//                 }
//             };

//             if !path.starts_with(&dir) {
//                 continue;
//             }

//             if let Ok(file) = fs::File::open(&path) {
//                 if let Ok(metadata) = file.metadata() {
//                     if metadata.is_file() {
//                         let file = InputFile::from(file);
//                         return Ok(SearchResult { path, file });
//                     }
//                 }
//             }
//         }

//         Err(io::Error::new(
//             io::ErrorKind::NotFound,
//             format!(
//                 "Could not find file {:?} along path \"{}\"",
//                 target.as_os_str(),
//                 self.to_string()
//             ),
//         ))
//     }

//     fn normalised(&self) -> Self {
//         Self {
//             path: self.path.iter().flat_map(|d| d.canonicalize()).collect(),
//         }
//     }
// }

// impl ToString for SearchPath {
//     fn to_string(&self) -> String {
//         self.path
//             .iter()
//             .map(|dir| dir.to_str().unwrap_or("?"))
//             .collect::<Vec<_>>()
//             .join(":")
//     }
// }

#[derive(Debug)]
pub struct SearchResult {
    path: path::PathBuf,
    file: InputFile,
}

impl SearchResult {
    pub fn path(&self) -> &path::Path {
        &self.path
    }

    pub fn file(&mut self) -> &mut InputFile {
        &mut self.file
    }
}

impl TryFrom<&str> for SearchResult {
    type Error = Error;

    fn try_from(value: &str) -> Result<Self> {
        Ok(Self {
            path: path::PathBuf::from(value),
            file: InputFile::from(fs::File::open(value)?),
        })
    }
}

impl TryFrom<&ArgPath> for SearchResult {
    type Error = Error;

    fn try_from(value: &ArgPath) -> Result<Self> {
        Ok(match value {
            ArgPath::Path(p) => Self {
                path: path::PathBuf::from(p),
                file: InputFile::from(fs::File::open(p)?),
            },
            ArgPath::Stdio => Self {
                path: path::PathBuf::from("-"),
                file: InputFile::from(io::stdin()), // TODO(kcza): lock this!
            },
        })
    }
}

#[derive(Debug)]
pub enum InputFile {
    Stdin(io::Stdin),
    File(fs::File),
}

impl InputFile {
    pub fn len_hint(&self) -> Option<u64> {
        match self {
            Self::File(f) => f.metadata().ok().map(|m| m.len()),
            Self::Stdin(_) => None,
        }
    }
}

impl From<fs::File> for InputFile {
    fn from(f: fs::File) -> Self {
        Self::File(f)
    }
}

impl From<io::Stdin> for InputFile {
    fn from(stdin: io::Stdin) -> Self {
        Self::Stdin(stdin)
    }
}

impl Read for InputFile {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        match self {
            Self::Stdin(s) => s.read(buf),
            Self::File(f) => f.read(buf),
        }
    }
}

#[cfg(test)]
impl InputFile {
    fn stdin(&self) -> Option<&Stdin> {
        match self {
            Self::Stdin(s) => Some(s),
            _ => None,
        }
    }

    fn file(&self) -> Option<BufReader<&File>> {
        match self {
            Self::File(f) => Some(BufReader::new(f)),
            _ => None,
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    // mod search_path {
    //     use super::*;
    //     use std::io;

    //     #[test]
    //     fn search_path_from() {
    //         assert_eq!(
    //             SearchPath::from("foo:bar::baz"),
    //             SearchPath {
    //                 path: vec!["foo", "bar", "baz"].iter().map(|d| d.into()).collect()
    //             }
    //         );

    //         assert_eq!(
    //             SearchPath::from("foo:bar::baz".to_owned()),
    //             SearchPath {
    //                 path: vec!["foo", "bar", "baz"].iter().map(|d| d.into()).collect()
    //             }
    //         );

    //         assert_eq!(
    //             SearchPath::from(
    //                 vec!["foo", "bar", "baz"]
    //                     .iter()
    //                     .map(path::PathBuf::from)
    //                     .collect::<Vec<_>>()
    //             ),
    //             SearchPath {
    //                 path: vec!["foo", "bar", "baz"].iter().map(|d| d.into()).collect()
    //             }
    //         );
    //     }

    //     #[test]
    //     fn to_string() {
    //         let path = SearchPath::from("asdf:fdsa: ::q");
    //         assert_eq!(path.to_string(), "asdf:fdsa: :q");
    //     }

    //     fn make_file(tmppath: &path::Path, filepath: &str, content: &str) -> Result<(), io::Error> {
    //         let path = path::PathBuf::from(tmppath).join(filepath);

    //         if let Some(parent) = path.parent() {
    //             fs::create_dir_all(parent)?;
    //         }

    //         fs::write(path, content)
    //     }

    //     #[test]
    //     fn open() -> Result<(), io::Error> {
    //         let tmpdir = tempfile::tempdir()?;
    //         let tmppath = tmpdir.path().canonicalize()?;

    //         make_file(&tmppath, "a.txt", "a")?;
    //         make_file(&tmppath, "B/b.txt", "b")?;
    //         make_file(&tmppath, "C1/C2/c.txt", "c")?;
    //         make_file(&tmppath, "D/d.txt", "c")?;
    //         make_file(&tmppath, "x.txt", "x")?;

    //         let raw_path: Vec<path::PathBuf> = vec!["B", "C1", "D"]
    //             .iter()
    //             .map(|s| path::PathBuf::from(&tmppath).join(s))
    //             .collect();
    //         let path = SearchPath::from(raw_path).normalised();

    //         {
    //             let a = path.open(&tmppath, "a.txt");
    //             assert!(a.is_ok(), "{:?}", a);
    //             let mut content = String::new();
    //             let mut found = a.unwrap();
    //             assert_eq!(found.path, tmppath.join("a.txt"));
    //             found.file().read_to_string(&mut content)?;
    //             assert_eq!(content, "a");
    //         }

    //         {
    //             let b = path.open(&tmppath, "b.txt");
    //             assert!(b.is_ok(), "{:?}", b);
    //             let mut found = b.unwrap();
    //             assert_eq!(found.path, tmppath.join("B/b.txt"));
    //             let mut content = String::new();
    //             found.file().read_to_string(&mut content)?;
    //             assert_eq!(content, "b");
    //         }

    //         {
    //             let c = path.open(&tmppath, "C2/c.txt");
    //             assert!(c.is_ok());
    //             let mut found = c.unwrap();
    //             assert_eq!(found.path, tmppath.join("C1/C2/c.txt"));
    //             let mut content = String::new();
    //             found.file().read_to_string(&mut content)?;
    //             assert_eq!(content, "c");
    //         }

    //         {
    //             let c = path.open(&tmppath, "D/d.txt");
    //             assert!(c.is_ok());
    //             let mut found = c.unwrap();
    //             assert_eq!(found.path, tmppath.join("D/d.txt"));
    //             let mut content = String::new();
    //             found.file().read_to_string(&mut content)?;
    //             assert_eq!(content, "c");
    //         }

    //         {
    //             let abs_path = tmppath.join("a.txt");
    //             let abs_result =
    //                 path.open(&tmppath, path::PathBuf::from(&abs_path).canonicalize()?);
    //             assert!(abs_result.is_err());
    //             let err = abs_result.unwrap_err();
    //             assert_eq!(err.kind(), io::ErrorKind::InvalidInput);
    //             assert_eq!(
    //                 err.to_string(),
    //                 format!("Absolute paths are forbidden: got {:?}", abs_path,)
    //             );
    //         }

    //         {
    //             let dir_result = path.open(&tmppath, "D");
    //             assert!(dir_result.is_err());
    //             let err = dir_result.unwrap_err();
    //             assert_eq!(err.kind(), io::ErrorKind::NotFound);
    //             assert_eq!(
    //                 err.to_string(),
    //                 format!(
    //                     "Could not find file \"D\" along path \"{}\"",
    //                     path.to_string()
    //                 )
    //             );
    //         }

    //         {
    //             let dir_result = path.open(&tmppath, "C2");
    //             assert!(dir_result.is_err());
    //             let err = dir_result.unwrap_err();
    //             assert_eq!(err.kind(), io::ErrorKind::NotFound);
    //             assert_eq!(
    //                 err.to_string(),
    //                 format!(
    //                     "Could not find file \"C2\" along path \"{}\"",
    //                     path.to_string()
    //                 )
    //             );
    //         }

    //         {
    //             let inaccessible = path.open(&tmppath, "c.txt");
    //             assert!(inaccessible.is_err());
    //             let err = inaccessible.unwrap_err();
    //             assert_eq!(err.kind(), io::ErrorKind::NotFound);
    //             assert_eq!(
    //                 err.to_string(),
    //                 format!(
    //                     "Could not find file \"c.txt\" along path \"{}\"",
    //                     path.to_string()
    //                 )
    //             );
    //         }

    //         {
    //             let inaccessible = path.open(&tmppath, "../a.txt");
    //             assert!(inaccessible.is_err());
    //             let abs_file = inaccessible.unwrap_err();
    //             assert_eq!(abs_file.kind(), io::ErrorKind::NotFound);
    //             assert_eq!(
    //                 abs_file.to_string(),
    //                 format!(
    //                     "Could not find file \"../a.txt\" along path \"{}\"",
    //                     path.to_string()
    //                 )
    //             );
    //         }

    //         {
    //             let non_existent = path.open(&tmppath, "non-existent.txt");
    //             assert!(non_existent.is_err());
    //             let non_existent = non_existent.unwrap_err();
    //             assert_eq!(non_existent.kind(), io::ErrorKind::NotFound);
    //             assert_eq!(
    //                 non_existent.to_string(),
    //                 format!(
    //                     "Could not find file \"non-existent.txt\" along path \"{}\"",
    //                     path.to_string()
    //                 )
    //             );
    //         }

    //         Ok(())
    //     }
    // }

    mod search_result {
        use super::*;
        use io::Write;

        #[test]
        fn fields() -> io::Result<()> {
            let tmpdir = tempfile::tempdir()?;
            let path = tmpdir.path().join("fields.txt");
            let mut file = fs::File::create(&path)?;
            file.write_all(b"file-content")?;

            let file = fs::File::open(&path)?;
            let mut s = SearchResult {
                path: path.clone(),
                file: InputFile::from(file),
            };

            assert_eq!(s.path, path);
            assert_eq!(
                {
                    let mut buf = String::new();
                    s.file().file().unwrap().read_to_string(&mut buf)?;
                    buf
                },
                "file-content"
            );

            Ok(())
        }

        #[test]
        fn from_str() -> Result<()> {
            let src = "from.txt";

            let tmpdir = tempfile::tempdir()?;
            let path = tmpdir.path().join(src);
            let mut file = fs::File::create(&path)?;
            file.write_all(b"file-content")?;

            let mut s = SearchResult::try_from(path.to_str().unwrap())?;
            assert_eq!(s.path, path);
            assert_eq!(
                {
                    let mut buf = String::new();
                    s.file().file().unwrap().read_to_string(&mut buf)?;
                    buf
                },
                "file-content",
            );

            Ok(())
        }

        #[test]
        fn from_arg_path() -> Result<()> {
            let src = "from.txt";

            let tmpdir = tempfile::tempdir()?;
            let path = tmpdir.path().join(src);
            let mut file = fs::File::create(&path)?;
            file.write_all(b"file-content")?;

            {
                let a = ArgPath::Path(path);
                let mut s: SearchResult = a.as_ref().try_into()?;
                assert_eq!(a.path().unwrap(), s.path);
                assert_eq!(
                    {
                        let mut buf = String::new();
                        s.file().file().unwrap().read_to_string(&mut buf)?;
                        buf
                    },
                    "file-content",
                );
            }

            {
                let a = ArgPath::Stdio;
                let s: SearchResult = a.as_ref().try_into()?;
                assert_eq!(s.path, path::PathBuf::from("-"));
                assert!(s.file.stdin().is_some());
            }

            Ok(())
        }

        #[test]
        fn len_hint() -> io::Result<()> {
            let tmpdir = tempfile::tempdir()?;
            let path = tmpdir.path().join("file.txt");
            let mut file = fs::File::create(&path)?;
            file.write_all(b"1234567890")?;

            assert_eq!(InputFile::from(io::stdin()).len_hint(), None);
            assert_eq!(InputFile::from(File::open(path)?).len_hint(), Some(10));

            Ok(())
        }
    }
}
