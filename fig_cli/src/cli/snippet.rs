use fuzzy_matcher::clangd::ClangdMatcher;
use reqwest::Method;

use crate::util::api::request;
use serde::{Serialize, Deserialize};

#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "type")]
#[serde(rename_all = "camelCase")]
enum Generator {
    #[serde(rename_all = "camelCase")]
    Named {
        name: String,
    },
    #[serde(rename_all = "camelCase")]
    Script {
        script: String,
    }
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
    Text {
        placeholder: Option<String>,
    },
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
    parameter_type: ParameterType
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

#[tokio::test]
pub async fn test() {
    let snippets: Vec<Snippet> = request(Method::GET, "/snippets", None, true).await.unwrap();
}
