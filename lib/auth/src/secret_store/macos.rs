use security_framework::os::macos::keychain::SecKeychain;

use crate::Result;

/// The account name is not used.
const ACCOUNT: &str = "";

/// errSecItemNotFound
const NOT_FOUND: i32 = -25300;

pub struct SecretStoreImpl {
    keychain: SecKeychain,
}

impl SecretStoreImpl {
    pub fn load() -> Result<Self> {
        match SecKeychain::open("login.keychain") {
            Ok(keychain) => Ok(Self { keychain }),
            Err(_) => {
                let keychain = SecKeychain::default()?;
                Ok(Self { keychain })
            },
        }
    }

    pub fn set(&self, key: &str, password: &str) -> Result<()> {
        if let Ok((_, mut item)) = self.keychain.find_generic_password(key, ACCOUNT) {
            item.set_password(password.as_bytes())?;
        } else {
            self.keychain.add_generic_password(key, ACCOUNT, password.as_bytes())?;
        }
        Ok(())
    }

    pub fn get(&self, key: &str) -> Result<Option<String>> {
        let password = match self.keychain.find_generic_password(key, ACCOUNT) {
            Ok((password, _)) => password,
            Err(err) if err.code() == NOT_FOUND => return Ok(None),
            Err(err) => return Err(err.into()),
        };
        Ok(Some(String::from_utf8(password.as_ref().to_vec())?))
    }

    pub fn delete(&self, key: &str) -> Result<()> {
        if let Ok((_, item)) = self.keychain.find_generic_password(key, ACCOUNT) {
            item.delete();
        }
        Ok(())
    }
}
