use serde::{Deserialize, Serialize};
use thiserror::Error;
use tokio::io;

#[derive(Debug, Error)]
pub enum Error {
    #[error("Io error: {0}")]
    IoError(#[from] io::Error),
    #[error("Json error: {0}")]
    JsonError(#[from] serde_json::Error),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum Availability {
    WhenFocused,
    Always,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct KeyBinding {
    pub identifier: String,
    pub name: Option<String>,
    pub description: Option<String>,
    pub category: Option<String>,
    pub availability: Option<Availability>,
    pub default_bindings: Option<Vec<String>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(transparent)]
pub struct KeyBindings(pub Vec<KeyBinding>);

impl KeyBindings {
    pub fn load() -> Result<Self, Error> {
        let path = fig_directories::fig_dir()
            .map(|dir| dir.join("apps").join("autocomplete").join("actions.json"))
            .unwrap();
        Ok(serde_json::from_reader(std::fs::File::open(&path)?)?)
    }

    pub fn load_hardcoded() -> Self {
        serde_json::from_str(&include_str!("test/actions.json"))
            .expect("Unable to load hardcoded actions")
    }
}

#[cfg(test)]
mod test {
    use super::*;

    const TEST_JSON: &str = include_str!("test/actions.json");

    #[test]
    fn test_load() {
        KeyBindings::load().unwrap();
    }

    #[test]
    fn test_load_json() {
        let json = serde_json::from_str::<KeyBindings>(TEST_JSON).unwrap();
        assert_eq!(json.0.len(), 18);

        assert_eq!(json.0[0].identifier, "insertSelected");
        assert_eq!(json.0[0].name, Some("Insert selected".to_string()));
        assert_eq!(
            json.0[0].description,
            Some("Insert selected suggestion".to_string())
        );
        assert_eq!(json.0[0].category, Some("Insertion".to_string()));
        assert_eq!(json.0[0].availability, Some(Availability::WhenFocused));
        assert_eq!(json.0[0].default_bindings, Some(vec!["enter".to_string()]));
    }
}
