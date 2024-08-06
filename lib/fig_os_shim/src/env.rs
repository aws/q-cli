use std::collections::HashMap;
use std::env::{
    self,
    VarError,
};
use std::ffi::{
    OsStr,
    OsString,
};
use std::path::PathBuf;

#[derive(Debug, Clone, Default)]
pub struct Env(inner::Inner);

mod inner {
    use std::collections::HashMap;

    #[derive(Debug, Clone, Default)]
    pub(super) enum Inner {
        #[default]
        Real,
        Fake(HashMap<String, String>),
    }
}

impl Env {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn get<K: AsRef<str>>(&self, key: K) -> Result<String, VarError> {
        use inner::Inner;
        match &self.0 {
            Inner::Real => env::var(key.as_ref()),
            Inner::Fake(map) => map.get(key.as_ref()).cloned().ok_or(VarError::NotPresent),
        }
    }

    pub fn get_os<K: AsRef<OsStr>>(&self, key: K) -> Option<OsString> {
        use inner::Inner;
        match &self.0 {
            Inner::Real => env::var_os(key.as_ref()),
            Inner::Fake(map) => map.get(key.as_ref().to_str()?).cloned().map(OsString::from),
        }
    }

    pub fn home(&self) -> Option<PathBuf> {
        match &self.0 {
            inner::Inner::Real => dirs::home_dir(),
            inner::Inner::Fake(map) => map.get("HOME").map(PathBuf::from),
        }
    }

    /// Create a fake process environment from a slice of tuples.
    pub fn from_slice(vars: &[(&str, &str)]) -> Self {
        use inner::Inner;
        let map: HashMap<_, _> = vars.iter().map(|(k, v)| ((*k).to_owned(), (*v).to_owned())).collect();
        Self(Inner::Fake(map))
    }
}

#[cfg(test)]
mod tests {
    use std::path::Path;

    use super::*;

    #[test]
    fn test_get() {
        let env = Env::default();
        assert!(env.home().is_some());
        assert!(env.get("PATH").is_ok());
        assert!(env.get("NON_EXISTENT").is_err());

        let env = Env::from_slice(&[("HOME", "/home/user"), ("PATH", "/bin:/usr/bin")]);
        assert_eq!(env.home().unwrap(), Path::new("/home/user"));
        assert_eq!(env.get("PATH").unwrap(), "/bin:/usr/bin");
        assert!(env.get("NON_EXISTENT").is_err());
    }
}
