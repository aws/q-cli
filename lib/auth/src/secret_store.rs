use security_framework::base::Error;
use security_framework::os::macos::keychain::SecKeychain;

/// The account name is not used.
const ACCOUNT: &str = "";
const NOT_FOUND: i32 = -25300; // errSecItemNotFound

fn keychain() -> Result<SecKeychain, Error> {
    match SecKeychain::open("login.keychain") {
        Ok(keychain) => Ok(keychain),
        Err(_) => SecKeychain::default(),
    }
}

pub fn set_secret(key: &str, password: &str) -> Result<(), Error> {
    let keychain = keychain()?;
    if let Ok((_, item)) = keychain.find_generic_password(&key, ACCOUNT) {
        item.delete();
    }
    keychain.add_generic_password(&key, ACCOUNT, password.as_bytes())?;
    Ok(())
}

pub fn get_secret(key: &str) -> Result<Option<String>, Error> {
    let keychain = keychain()?;
    let password = match keychain.find_generic_password(&key, ACCOUNT) {
        Ok((password, _)) => password,
        Err(err) if err.code() == NOT_FOUND => return Ok(None),
        Err(err) => return Err(err),
    };
    Ok(String::from_utf8(password.as_ref().to_vec()).ok())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_set_password() {
        set_secret("XXXX", "test").unwrap();
        println!("{}", get_secret("XXXX").unwrap().unwrap());
    }
}
