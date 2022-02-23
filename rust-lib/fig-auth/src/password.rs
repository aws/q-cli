use rand::Rng;

/// Generates a password of the given length
///
/// The password is guaranteed to include at least one lowercase letter,
/// one uppercase letter, one digit, and one special character.
///
/// Length must be greater than or equal to 4
pub fn generate_password(length: usize) -> String {
    assert!(length >= 4);

    let special = r#"^$*.[]{}()?-"!@#%&/\,><':;|_~`+="#.chars().collect::<Vec<_>>();
    let alphanum = "abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789"
        .chars()
        .collect::<Vec<_>>();
    let all_characters = special.iter().chain(alphanum.iter()).collect::<Vec<_>>();

    let mut rng = rand::thread_rng();

    loop {
        let mut password = String::with_capacity(length);

        for _ in 0..length {
            password.push(*all_characters[rng.gen_range(0..all_characters.len())]);
        }

        // Check for lowercase, uppercase, digit, and special character
        if password.chars().any(|c| c.is_numeric())
            && password.chars().any(|c| special.contains(&c))
            && password.chars().any(|c| c.is_ascii_uppercase())
            && password.chars().any(|c| c.is_ascii_lowercase())
        {
            return password;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_password() {
        for _ in 0..64 {
            let password = generate_password(32);
            assert!(password.chars().any(|c| c.is_numeric()));
            assert!(password.chars().any(|c| c.is_ascii_uppercase()));
            assert!(password.chars().any(|c| c.is_ascii_lowercase()));
            assert!(password.chars().any(|c| c.is_ascii_punctuation()));
        }
    }
}
