use std::collections::HashMap;
use std::io;
use std::path::{
    Path,
    PathBuf,
};
use std::sync::{
    Arc,
    Mutex,
};

use tokio::fs;

#[derive(Debug, Clone, Default)]
pub struct Fs(inner::Inner);

mod inner {
    use std::collections::HashMap;
    use std::path::PathBuf;
    use std::sync::{
        Arc,
        Mutex,
    };

    #[derive(Debug, Clone, Default)]
    pub(super) enum Inner {
        #[default]
        Real,
        Fake(Arc<Mutex<HashMap<PathBuf, Vec<u8>>>>),
    }
}

impl Fs {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn new_fake() -> Self {
        Self(inner::Inner::Fake(Arc::new(Mutex::new(HashMap::new()))))
    }

    pub fn from_slice(vars: &[(&str, &str)]) -> Self {
        use inner::Inner;
        let map: HashMap<_, _> = vars
            .iter()
            .map(|(k, v)| (PathBuf::from(k), v.as_bytes().to_vec()))
            .collect();
        Self(Inner::Fake(Arc::new(Mutex::new(map))))
    }

    pub async fn create_dir(&self, path: impl AsRef<Path>) -> io::Result<()> {
        use inner::Inner;
        match &self.0 {
            Inner::Real => fs::create_dir(path).await,
            Inner::Fake(_) => Err(io::Error::new(io::ErrorKind::Other, "unimplemented")),
        }
    }

    pub async fn create_dir_all(&self, path: impl AsRef<Path>) -> io::Result<()> {
        use inner::Inner;
        match &self.0 {
            Inner::Real => fs::create_dir_all(path).await,
            Inner::Fake(_) => Err(io::Error::new(io::ErrorKind::Other, "unimplemented")),
        }
    }

    pub async fn read(&self, path: impl AsRef<Path>) -> io::Result<Vec<u8>> {
        use inner::Inner;
        match &self.0 {
            Inner::Real => fs::read(path).await,
            Inner::Fake(map) => {
                let Ok(lock) = map.lock() else {
                    return Err(io::Error::new(io::ErrorKind::Other, "poisoned lock"));
                };
                let Some(data) = lock.get(path.as_ref()) else {
                    return Err(io::Error::new(io::ErrorKind::NotFound, "not found"));
                };
                Ok(data.clone())
            },
        }
    }

    pub async fn read_to_string(&self, path: impl AsRef<Path>) -> io::Result<String> {
        use inner::Inner;
        match &self.0 {
            Inner::Real => fs::read_to_string(path).await,
            Inner::Fake(map) => {
                let Ok(lock) = map.lock() else {
                    return Err(io::Error::new(io::ErrorKind::Other, "poisoned lock"));
                };
                let Some(data) = lock.get(path.as_ref()) else {
                    return Err(io::Error::new(io::ErrorKind::NotFound, "not found"));
                };
                match String::from_utf8(data.clone()) {
                    Ok(string) => Ok(string),
                    Err(err) => Err(io::Error::new(io::ErrorKind::InvalidData, err)),
                }
            },
        }
    }

    pub async fn write(&self, path: impl AsRef<Path>, contents: impl AsRef<[u8]>) -> io::Result<()> {
        use inner::Inner;
        match &self.0 {
            Inner::Real => fs::write(path, contents).await,
            Inner::Fake(map) => {
                let Ok(mut lock) = map.lock() else {
                    return Err(io::Error::new(io::ErrorKind::Other, "poisoned lock"));
                };
                lock.insert(path.as_ref().to_owned(), contents.as_ref().to_owned());
                Ok(())
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_real() {
        let dir = tempfile::tempdir().unwrap();
        let fs = Fs::new();

        fs.create_dir(dir.path().join("create_dir")).await.unwrap();
        fs.create_dir_all(dir.path().join("create/dir/all")).await.unwrap();
        fs.write(dir.path().join("write"), b"write").await.unwrap();
        fs.read(dir.path().join("write")).await.unwrap();
        fs.read_to_string(dir.path().join("write")).await.unwrap();
    }
}
