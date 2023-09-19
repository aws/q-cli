use std::str::FromStr;

pub use crate::proto::local::*;

impl FromStr for devtools_command::Window {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase() {
            s if s == "autocomplete" => Ok(devtools_command::Window::DevtoolsAutocomplete),
            s if s == "dashboard" => Ok(devtools_command::Window::DevtoolsDashboard),
            _ => Err("unknown devtools window".to_owned()),
        }
    }
}
