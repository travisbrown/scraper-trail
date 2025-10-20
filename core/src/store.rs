use std::path::{Path, PathBuf};

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("I/O error")]
    Io(#[from] std::io::Error),
    #[error("JSON error")]
    Json(#[from] serde_json::Error),
    #[error("Exchange error")]
    Exchange(#[from] crate::exchange::Error),
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Store {
    pub base: PathBuf,
}

impl Store {
    pub fn new<P: AsRef<Path>>(base: P) -> Self {
        Self {
            base: base.as_ref().to_path_buf(),
        }
    }

    pub fn paths(&self, reverse: bool) -> Result<Vec<PathBuf>, std::io::Error> {
        let mut paths = std::fs::read_dir(&self.base)?
            .map(|entry| entry.map(|entry| entry.path()))
            .collect::<Result<Vec<_>, _>>()?;

        paths.sort();

        if reverse {
            paths.reverse();
        }

        Ok(paths)
    }

    pub fn contents(&self, reverse: bool) -> Result<Contents, std::io::Error> {
        Ok(Contents {
            // We put the paths in reverse order, since we'll be popping them off the `Vec`.
            paths: self.paths(!reverse)?,
        })
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Contents {
    paths: Vec<PathBuf>,
}

impl Iterator for Contents {
    type Item = (PathBuf, Result<String, Error>);

    fn next(&mut self) -> Option<Self::Item> {
        self.paths.pop().map(|path| {
            let contents = std::fs::read_to_string(&path).map_err(Error::from);

            (path, contents)
        })
    }
}
