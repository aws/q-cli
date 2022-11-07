use serde::de::DeserializeOwned;

use crate::{
    JsonStore,
    Result,
    State,
};

pub fn init_global() -> Result<()> {
    State::load_into_global()
}

pub fn set_value(key: impl Into<String>, value: impl Into<serde_json::Value>) -> Result<()> {
    let mut state = State::load()?;
    state.set(key, value);
    state.save_to_file()?;
    Ok(())
}

pub fn remove_value(key: impl AsRef<str>) -> Result<()> {
    let mut state = State::load()?;
    state.remove(key);
    state.save_to_file()?;
    Ok(())
}

pub fn get_value(key: impl AsRef<str>) -> Result<Option<serde_json::Value>> {
    Ok(State::load()?.get(key).map(|v| v.clone()))
}

pub fn get<T: DeserializeOwned>(key: impl AsRef<str>) -> Result<Option<T>> {
    let state = State::load()?;
    let v = state.get(key);
    match v.as_deref() {
        Some(value) => Ok(Some(serde_json::from_value(value.clone())?)),
        None => Ok(None),
    }
}

pub fn get_bool(key: impl AsRef<str>) -> Result<Option<bool>> {
    Ok(State::load()?.get_bool(key))
}

pub fn get_bool_or(key: impl AsRef<str>, default: bool) -> bool {
    get_bool(key).ok().flatten().unwrap_or(default)
}

pub fn get_string(key: impl AsRef<str>) -> Result<Option<String>> {
    Ok(State::load()?.get_string(key))
}

pub fn get_string_or(key: impl AsRef<str>, default: impl Into<String>) -> String {
    get_string(key).ok().flatten().unwrap_or_else(|| default.into())
}

pub fn get_int(key: impl AsRef<str>) -> Result<Option<i64>> {
    Ok(State::load()?.get_int(key))
}

pub fn get_int_or(key: impl AsRef<str>, default: i64) -> i64 {
    get_int(key).ok().flatten().unwrap_or(default)
}

pub fn create_anonymous_id() -> Result<String> {
    let anonymous_id = uuid::Uuid::new_v4().as_hyphenated().to_string();
    set_value("anonymousId", anonymous_id.clone())?;
    Ok(anonymous_id)
}

pub fn get_or_create_anonymous_id() -> Result<String> {
    if let Ok(Some(anonymous_id)) = get_string("anonymousId") {
        return Ok(anonymous_id);
    }

    create_anonymous_id()
}

//     /// General read/write state test
//     #[fig_test::test]
//     fn test_state() -> Result<()> {
//         let path = tempfile::tempdir().unwrap().into_path().join("local.json");
//         std::env::set_var("FIG_DIRECTORIES_STATE_PATH", &path);

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

//         assert!(get_string_or("string", "hi") == "hi");
//         set_value("string", "hi").unwrap();
//         assert!(get_string("string").unwrap().unwrap() == "hi");

//         assert!(get_int_or("int", 32) == 32);
//         set_value("int", 32).unwrap();
//         assert!(get_int("int").unwrap().unwrap() == 32);

//         Ok(())
//     }
// }
