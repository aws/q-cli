use anyhow::{
    anyhow,
    Result,
};
use reqwest::Method;
use serde::{
    Deserialize,
    Serialize,
};

use crate::util::api::request;

#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "type")]
#[serde(rename_all = "camelCase")]
enum Generator {
    #[serde(rename_all = "camelCase")]
    Named { name: String },
    #[serde(rename_all = "camelCase")]
    Script { script: String },
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "type", content = "typeData")]
#[serde(rename_all = "camelCase")]
enum ParameterType {
    #[serde(rename_all = "camelCase")]
    Checkbox {
        true_value_substitution: String,
        false_value_substitution: String,
    },
    #[serde(rename_all = "camelCase")]
    Text { placeholder: Option<String> },
    #[serde(rename_all = "camelCase")]
    Selector {
        placeholder: Option<String>,
        suggestions: Option<Vec<String>>,
        generators: Vec<Generator>,
    },
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct Parameter {
    name: String,
    display_name: Option<String>,
    description: Option<String>,
    #[serde(flatten)]
    parameter_type: ParameterType,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct Snippet {
    name: String,
    display_name: Option<String>,
    description: Option<String>,
    tags: Option<Vec<String>>,
    parameters: Vec<Parameter>,
}

pub async fn execute(name: Option<String>) -> Result<()> {
    let snippets: Vec<Snippet> = request(Method::GET, "/snippets", None, true).await?;

    let snippet = match name {
        Some(name) => match snippets.iter().find(|snippet| snippet.name == name) {
            Some(snippet) => snippet,
            None => return Err(anyhow!("No snippet with name: {}", name)),
        },
        None => {
            let snippet_names: Vec<&str> = snippets.iter().map(|snippet| snippet.name.as_ref()).collect();
            let selection = dialoguer::FuzzySelect::with_theme(&crate::util::dialoguer_theme())
                .items(&snippet_names)
                .default(0)
                .interact()
                .unwrap();
            &snippets[selection]
        },
    };

    // for param in &snippet.parameters {
    //    match &param.parameter_type {
    //        ParameterType::Checkbox { true_value_substitution, false_value_substitution } => todo!(),
    //        ParameterType::Text { placeholder } => todo!(),
    //        ParameterType::Selector { placeholder, suggestions, generators } => todo!(),
    //    }
    //}

    Ok(())
}
