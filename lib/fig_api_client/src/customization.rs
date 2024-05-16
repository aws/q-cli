use amzn_codewhisperer_client::types::Customization as CwCustomization;
use serde::{
    Deserialize,
    Serialize,
};

const CUSTOMIZATION_STATE_KEY: &str = "api.selectedCustomization";

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]

pub struct Customization {
    pub arn: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
}

impl Customization {
    /// Load the currently selected customization from state
    pub fn load_selected() -> Result<Option<Self>, fig_settings::Error> {
        fig_settings::state::get(CUSTOMIZATION_STATE_KEY)
    }

    /// Save the currently selected customization to state
    pub fn save_selected(&self) -> Result<(), fig_settings::Error> {
        fig_settings::state::set_value(CUSTOMIZATION_STATE_KEY, serde_json::to_value(self)?)
    }

    /// Delete the currently selected customization from state
    pub fn delete_selected() -> Result<(), fig_settings::Error> {
        fig_settings::state::remove_value(CUSTOMIZATION_STATE_KEY)
    }
}

impl From<Customization> for CwCustomization {
    fn from(Customization { arn, name, description }: Customization) -> Self {
        CwCustomization::builder()
            .arn(arn)
            .set_name(name)
            .set_description(description)
            .build()
            .expect("Failed to build CW Customization")
    }
}

impl From<CwCustomization> for Customization {
    fn from(cw_customization: CwCustomization) -> Self {
        Customization {
            arn: cw_customization.arn,
            name: cw_customization.name,
            description: cw_customization.description,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_customization_from_impls() {
        let cw_customization = CwCustomization::builder()
            .arn("arn")
            .name("name")
            .description("description")
            .build()
            .unwrap();

        let custom_from_cw: Customization = cw_customization.into();
        let cw_from_custom: CwCustomization = custom_from_cw.into();

        assert_eq!(cw_from_custom.arn, "arn");
        assert_eq!(cw_from_custom.name, Some("name".into()));
        assert_eq!(cw_from_custom.description, Some("description".into()));

        let cw_customization = CwCustomization::builder().arn("arn").build().unwrap();

        let custom_from_cw: Customization = cw_customization.into();
        let cw_from_custom: CwCustomization = custom_from_cw.into();

        assert_eq!(cw_from_custom.arn, "arn");
        assert_eq!(cw_from_custom.name, None);
        assert_eq!(cw_from_custom.description, None);
    }

    #[test]
    fn test_customization_save_load() {
        let old_value = Customization::load_selected().unwrap();

        let new_value = Customization {
            arn: "arn".into(),
            name: Some("name".into()),
            description: Some("description".into()),
        };

        new_value.save_selected().unwrap();
        let loaded_value = Customization::load_selected().unwrap();
        assert_eq!(loaded_value, Some(new_value));

        Customization::delete_selected().unwrap();
        if let Some(old_value) = old_value {
            old_value.save_selected().unwrap();
        }
    }

    #[test]
    fn test_customization_serde() {
        let customization = Customization {
            arn: "arn".into(),
            name: Some("name".into()),
            description: Some("description".into()),
        };

        let serialized = serde_json::to_string(&customization).unwrap();
        assert_eq!(serialized, r#"{"arn":"arn","name":"name","description":"description"}"#);

        let deserialized: Customization = serde_json::from_str(&serialized).unwrap();
        assert_eq!(deserialized, customization);

        let customization = Customization {
            arn: "arn".into(),
            name: None,
            description: None,
        };

        let serialized = serde_json::to_string(&customization).unwrap();
        assert_eq!(serialized, r#"{"arn":"arn"}"#);

        let deserialized: Customization = serde_json::from_str(&serialized).unwrap();
        assert_eq!(deserialized, customization);
    }
}
