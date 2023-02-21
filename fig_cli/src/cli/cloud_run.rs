use bstr::ByteSlice;
use fig_request::Method;

pub async fn cloud_run(namespace: String, name: String) -> eyre::Result<()> {
    let token = std::process::Command::new("fly")
        .arg("auth")
        .arg("token")
        .output()
        .expect("failed to execute process")
        .stdout
        .to_str()
        .unwrap()
        .trim()
        .to_owned();

    let text: serde_json::Value = fig_request::Request::new_with_url(
        Method::POST,
        "http://localhost:4280/v1/apps/fig-user-functions/machines"
            .try_into()
            .unwrap(),
    )
    .custom_token(token.clone())
    .body_json(serde_json::json!(
        {
            // "name": "fig-user-functions",
            "config": {
                "image": "grant0417/fig-ubuntu",
                    "init": {
                        "cmd": ["fig", "run", format!("@{}/{}", namespace, name)],
                    },
                    "env": {
                        "FIG_TOKEN": "figapi_zQAFazrCFUGxBHiZFqwldUVXLUZQhYZV"
                    },
                    "services": [
                        {
                            "internal_port": 22,
                            "ports": [
                                {
                                    "port": 22,
                                    "protocol": "tcp"
                                }
                            ],
                            "protocol": "tcp",
                        }
                    ]
                }
          }
    ))
    .send()
    .await
    .unwrap()
    .json()
    .await
    .unwrap();

    println!("Starting machine");
    println!("{:#}", text);

    let machine_id = text["id"].as_str().unwrap();

    let _text: serde_json::Value = fig_request::Request::new_with_url(
        Method::GET,
        url::Url::parse(&format!(
            "http://localhost:4280/v1/apps/fig-user-functions/machines/{machine_id}/wait"
        ))
        .unwrap(),
    )
    .custom_token(token)
    .send()
    .await
    .unwrap()
    .json()
    .await
    .unwrap();

    println!("Machine started");

    return Ok(());
}
