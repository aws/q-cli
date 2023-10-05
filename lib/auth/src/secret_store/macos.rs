use security_framework::os::macos::keychain::SecKeychain;

use crate::Result;

/// The account name is not used.
const ACCOUNT: &str = "";

/// [`errSecItemNotFound`](https://developer.apple.com/documentation/security/1542001-security_framework_result_codes/errsecitemnotfound)
const ERR_SEC_ITEM_NOT_FOUND: i32 = -25300;

pub struct SecretStoreImpl {
    keychain: SecKeychain,
}

impl SecretStoreImpl {
    pub fn load() -> Result<Self> {
        match SecKeychain::open("login.keychain") {
            Ok(keychain) => Ok(Self { keychain }),
            Err(err) => match SecKeychain::default() {
                Ok(keychain) => Ok(Self { keychain }),
                Err(_) => Err(err.into()),
            },
        }
    }

    /// Sets the `key` to `password` on the keychain, this will override any existing value
    pub fn set(&self, key: &str, password: &str) -> Result<()> {
        if let Ok((_, mut item)) = self.keychain.find_generic_password(key, ACCOUNT) {
            item.set_password(password.as_bytes())?;
        } else {
            self.keychain.add_generic_password(key, ACCOUNT, password.as_bytes())?;
        }
        Ok(())
    }

    /// Returns the password for the `key`
    ///
    /// If not found the result will be `Ok(None)`, other errors will be returned
    pub fn get(&self, key: &str) -> Result<Option<String>> {
        match self.keychain.find_generic_password(key, ACCOUNT) {
            Ok((password, _)) => Ok(Some(String::from_utf8(password.as_ref().to_vec())?)),
            Err(err) if err.code() == ERR_SEC_ITEM_NOT_FOUND => Ok(None),
            Err(err) => Err(err.into()),
        }
    }

    /// Deletes the `key` from the keychain
    pub fn delete(&self, key: &str) -> Result<()> {
        if let Ok((_, item)) = self.keychain.find_generic_password(key, ACCOUNT) {
            item.delete();
        }
        Ok(())
    }
}
