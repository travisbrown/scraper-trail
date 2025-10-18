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

    #[must_use]
    pub fn contents(&self, reverse: bool) -> Contents<'_> {
        Contents {
            base: Some(&self.base),
            reverse,
            paths: vec![],
        }
    }
}

pub struct Contents<'a> {
    base: Option<&'a Path>,
    reverse: bool,
    paths: Vec<PathBuf>,
}

impl Iterator for Contents<'_> {
    type Item = Result<String, Error>;

    fn next(&mut self) -> Option<Self::Item> {
        match self.base.take() {
            Some(base) => match std::fs::read_dir(base).and_then(|entries| {
                entries
                    .map(|entry| entry.map(|entry| entry.path()))
                    .collect::<Result<Vec<_>, _>>()
            }) {
                Ok(mut paths) => {
                    paths.sort();

                    // We put the paths in reverse order, since we'll be popping them off the `Vec`.
                    if !self.reverse {
                        paths.reverse();
                    }

                    self.paths = paths;
                    self.next()
                }
                Err(error) => Some(Err(Error::from(error))),
            },
            None => self
                .paths
                .pop()
                .map(|path| std::fs::read_to_string(path).map_err(Error::from)),
        }
    }
}
