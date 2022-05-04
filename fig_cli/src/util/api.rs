use anyhow::{bail, Result};

pub async fn handle_fig_response(resp: reqwest::Response) -> Result<reqwest::Response> {
    if resp.status().is_success() {
        Ok(resp)
    } else {
        let err = resp.error_for_status_ref().err();
        macro_rules! print_err {
            () => {{
                match err {
                    Some(err) => bail!(err),
                    None => bail!("Unknown error"),
                }
            }};
        }

        match resp.text().await {
            Ok(text) => match serde_json::from_str::<serde_json::Value>(&text) {
                Ok(json) => {
                    bail!(json
                        .get("error")
                        .and_then(|error| error.as_str())
                        .unwrap_or("Unknown error")
                        .to_string())
                }
                Err(_) => {
                    if !text.is_empty() {
                        bail!(text)
                    } else {
                        print_err!()
                    }
                }
            },
            Err(_) => print_err!(),
        }
    }
}
