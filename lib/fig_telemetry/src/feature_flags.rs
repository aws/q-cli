use fig_settings::state::get_or_create_anonymous_id;
use parking_lot::Mutex;
use serde::{
    Deserialize,
    Serialize,
};
use serde_json::{
    json,
    Map,
    Value,
};

use crate::Error;

const POSTHOG_PUBLIC_API_KEY: &str = "phc_yjURwnd3JlTn2LT3iSdXBuqjHrMPDagtkrpeP7KxCdK";

// https://posthog.com/docs/api/post-only-endpoints#feature-flags
const FEATURE_FLAG_URL: &str = "https://feature-flags.fig.io/decide?v=2";

static FEATURE_FLAG_GLOBAL: Mutex<Option<FeatureFlags>> = Mutex::new(None);

#[derive(Debug, Clone)]
pub struct FeatureFlags {
    pub anonymous_id: String,
    pub flags: Map<String, Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct DecideResponse {
    feature_flags: Map<String, Value>,
}

pub async fn get_feature_flags() -> Result<FeatureFlags, Error> {
    let anonymous_id = get_or_create_anonymous_id()?;

    match FEATURE_FLAG_GLOBAL {
        ref global if global.lock().is_some() => {
            let global = global.lock();
            if global.as_ref().unwrap().anonymous_id == anonymous_id {
                return Ok(global.as_ref().unwrap().clone());
            }
        },
        _ => {},
    }

    let body = json!({
        "api_key": POSTHOG_PUBLIC_API_KEY,
        "distinct_id": anonymous_id,
    });

    let resp = fig_request::client()
        .unwrap()
        .post(FEATURE_FLAG_URL)
        .json(&body)
        .send()
        .await
        .map_err(fig_request::Error::from)?;

    let resp: DecideResponse = resp.json().await.map_err(fig_request::Error::from)?;

    let flags = FeatureFlags {
        anonymous_id,
        flags: resp.feature_flags,
    };

    *FEATURE_FLAG_GLOBAL.lock() = Some(flags.clone());

    Ok(flags)
}

pub async fn get_feature_flag(key: impl AsRef<str>) -> Option<Value> {
    let flags = get_feature_flags().await.ok()?;
    flags.flags.get(key.as_ref()).cloned()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[ignore = "we dont want to create a new anonymous id every time we run tests"]
    #[tokio::test]
    async fn test_get_feature_flags() {
        dbg!(get_feature_flags().await.unwrap());
    }
}
