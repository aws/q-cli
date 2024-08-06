use crate::{
    Env,
    Fs,
};

pub trait EnvProvider {
    fn env(&self) -> &Env;
}

impl EnvProvider for Env {
    fn env(&self) -> &Env {
        self
    }
}

pub trait FsProvider {
    fn fs(&self) -> &Fs;
}

impl FsProvider for Fs {
    fn fs(&self) -> &Fs {
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_env_provider() {
        let env = Env::default();
        let env_provider = &env as &dyn EnvProvider;
        env_provider.env();
    }

    #[test]
    fn test_fs_provider() {
        let fs = Fs::default();
        let fs_provider = &fs as &dyn FsProvider;
        fs_provider.fs();
    }
}
