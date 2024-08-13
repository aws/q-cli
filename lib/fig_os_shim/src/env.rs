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

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Env(inner::Inner);

mod inner {
    use std::collections::HashMap;

    #[derive(Debug, Clone, Default, PartialEq, Eq)]
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

    /// Create a fake process environment from a slice of tuples.
    pub fn from_slice(vars: &[(&str, &str)]) -> Self {
        use inner::Inner;
        let map: HashMap<_, _> = vars.iter().map(|(k, v)| ((*k).to_owned(), (*v).to_owned())).collect();
        Self(Inner::Fake(map))
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

    pub fn in_cloudshell(&self) -> bool {
        self.get("AWS_EXECUTION_ENV")
            .map_or(false, |v| v.trim().eq_ignore_ascii_case("cloudshell"))
    }

    pub fn in_ssh(&self) -> bool {
        self.get("SSH_CLIENT").is_ok() || self.get("SSH_CONNECTION").is_ok() || self.get("SSH_TTY").is_ok()
    }
}

#[cfg(test)]
mod tests {
    use std::path::Path;

    use super::*;

    #[test]
    fn test_new() {
        let env = Env::new();
        assert_eq!(env, Env(inner::Inner::Real));

        let env = Env::default();
        assert_eq!(env, Env(inner::Inner::Real));
    }

    #[test]
    fn test_get() {
        let env = Env::new();
        assert!(env.home().is_some());
        assert!(env.get("PATH").is_ok());
        assert!(env.get_os("PATH").is_some());
        assert!(env.get("NON_EXISTENT").is_err());

        let env = Env::from_slice(&[("HOME", "/home/user"), ("PATH", "/bin:/usr/bin")]);
        assert_eq!(env.home().unwrap(), Path::new("/home/user"));
        assert_eq!(env.get("PATH").unwrap(), "/bin:/usr/bin");
        assert!(env.get_os("PATH").is_some());
        assert!(env.get("NON_EXISTENT").is_err());
    }

    #[test]
    fn test_in_envs() {
        let env = Env::from_slice(&[]);
        assert!(!env.in_cloudshell());
        assert!(!env.in_ssh());

        let env = Env::from_slice(&[("AWS_EXECUTION_ENV", "CloudShell"), ("SSH_CLIENT", "1")]);
        assert!(env.in_cloudshell());
        assert!(env.in_ssh());

        let env = Env::from_slice(&[("AWS_EXECUTION_ENV", "CLOUDSHELL\n")]);
        assert!(env.in_cloudshell());
        assert!(!env.in_ssh());
    }
}
