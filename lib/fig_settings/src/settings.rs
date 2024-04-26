use serde::de::DeserializeOwned;

use crate::{
    JsonStore,
    Result,
    Settings,
};

pub fn init_global() -> Result<()> {
    Settings::load_into_global()
}

/// Do not use this if you want to update remote settings, use
/// fig_api_client::settings::update
pub fn set_value(key: impl Into<String>, value: impl Into<serde_json::Value>) -> Result<()> {
    let mut settings = Settings::load()?;
    settings.set(key, value);
    settings.save_to_file()?;
    Ok(())
}

/// Do not use this if you want to update remote settings_path, use
/// fig_api_client::settings::delete
pub fn remove_value(key: impl AsRef<str>) -> Result<()> {
    let mut settings = Settings::load()?;
    settings.remove(&key);
    settings.save_to_file()?;
    Ok(())
}

pub fn get_value(key: impl AsRef<str>) -> Result<Option<serde_json::Value>> {
    Ok(Settings::load()?.get(key).map(|v| v.clone()))
}

pub fn get<T: DeserializeOwned>(key: impl AsRef<str>) -> Result<Option<T>> {
    let settings = Settings::load()?;
    let v = settings.get(key);
    match v.as_deref() {
        Some(value) => Ok(Some(serde_json::from_value(value.clone())?)),
        None => Ok(None),
    }
}

pub fn get_bool(key: impl AsRef<str>) -> Result<Option<bool>> {
    Ok(Settings::load()?.get_bool(key))
}

pub fn get_bool_or(key: impl AsRef<str>, default: bool) -> bool {
    get_bool(key).ok().flatten().unwrap_or(default)
}

pub fn get_string(key: impl AsRef<str>) -> Result<Option<String>> {
    Ok(Settings::load()?.get_string(key))
}

pub fn get_string_opt(key: impl AsRef<str>) -> Option<String> {
    get_string(key).ok().flatten()
}

pub fn get_string_or(key: impl AsRef<str>, default: String) -> String {
    get_string(key).ok().flatten().unwrap_or(default)
}

pub fn get_int(key: impl AsRef<str>) -> Result<Option<i64>> {
    Ok(Settings::load()?.get_int(key))
}

pub fn get_int_or(key: impl AsRef<str>, default: i64) -> i64 {
    get_int(key).ok().flatten().unwrap_or(default)
}

// #[cfg(test)]
// mod test {
//     use super::*;

//     /// General read/write settings test
//     #[fig_test::test]
//     fn test_settings() -> Result<()> {
//         let path = tempfile::tempdir().unwrap().into_path().join("local.json");
//         std::env::set_var("FIG_DIRECTORIES_SETTINGS_PATH", &path);

//         local_settings()?;
//         get_map()?;

//         assert!(get_value("test").unwrap().is_none());
//         assert!(get::<String>("test").unwrap().is_none());
//         set_value("test", "hello :)")?;
//         assert!(get_value("test").unwrap().is_some());
//         assert!(get::<String>("test").unwrap().is_some());
//         remove_value("test")?;
//         assert!(get_value("test").unwrap().is_none());
//         assert!(get::<String>("test").unwrap().is_none());

//         assert!(!get_bool_or("bool", false));
//         set_value("bool", true).unwrap();
//         assert!(get_bool("bool").unwrap().unwrap());

//         assert!(get_string_or("string", "hi".into()) == "hi");
//         set_value("string", "hi").unwrap();
//         assert!(get_string("string").unwrap().unwrap() == "hi");

//         assert!(get_int_or("int", 32) == 32);
//         set_value("int", 32).unwrap();
//         assert!(get_int("int").unwrap().unwrap() == 32);

//         Ok(())
//     }

//     /// Sanity test over product gates
//     #[fig_test::test]
//     fn test_product_gate() -> Result<()> {
//         product_gate("test_product", Some("hello"))?;
//         product_gate("test_product", None::<String>)?;
//         Ok(())
//     }
// }
