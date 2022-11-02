use std::time::Duration;

use serde::{
    Deserialize,
    Serialize,
};

use crate::utils::run_stdout;

#[derive(Serialize)]
struct TriggerPipelineRequest {
    branch: String,
    parameters: TriggerPipelineParameters,
}

#[derive(Serialize)]
struct TriggerPipelineParameters {
    build_targets: String,
    checkout: String,
    compat: bool,
}

#[derive(Deserialize)]
struct TriggerPipelineSuccess {
    id: String,
    number: u64,
}

#[derive(Deserialize)]
struct CircleError {
    message: String,
}
#[derive(Deserialize)]
struct PipelineWorkflows {
    items: Vec<Workflow>,
}

#[derive(Deserialize)]
struct Workflow {
    id: String,
    name: String,
}

pub async fn publish(build_targets: Vec<String>, dry: bool) -> eyre::Result<()> {
    if build_targets.is_empty() {
        eyre::bail!("Didn't specify any build targets");
    }

    let token = std::env::var("CIRCLECI_TOKEN")
        .expect("Make sure you're logged into your company Fig account and have dotfiles enabled");
    let client = fig_request::client().unwrap().clone();

    let commit_hash = run_stdout(&["git", "rev-parse", "HEAD"])?.trim().to_string();
    let current_branch = run_stdout(&["git", "rev-parse", "--abbrev-ref", "HEAD"])?
        .trim()
        .to_string();

    if dry {
        println!("commit_hash: {commit_hash}");
        println!("current_branch: {current_branch}");
        return Ok(());
    }

    let resp = client
        .post("https://circleci.com/api/v2/project/github/withfig/macos/pipeline")
        .header("Circle-Token", &token)
        .json(&TriggerPipelineRequest {
            branch: current_branch,
            parameters: TriggerPipelineParameters {
                build_targets: build_targets
                    .into_iter()
                    .map(|target| {
                        if target == "all" {
                            "macos,linux,windows".to_string()
                        } else {
                            target
                        }
                    })
                    .collect::<Vec<String>>()
                    .join(","),
                checkout: commit_hash,
                compat: false,
            },
        })
        .send()
        .await?;

    if resp.status().is_success() {
        let resp: TriggerPipelineSuccess = resp.json().await?;

        println!("pipeline started, getting build workflow...");

        let pipeline_number = resp.number;
        let mut success = false;

        let mut interval = tokio::time::interval(Duration::from_secs(1));
        interval.tick().await;

        'wait: for _ in 0..10 {
            let resp = client
                .get(format!("https://circleci.com/api/v2/pipeline/{}/workflow", resp.id))
                .header("Circle-Token", &token)
                .send()
                .await?;

            if resp.status().is_success() {
                let resp: PipelineWorkflows = resp.json().await?;

                for workflow in resp.items {
                    if workflow.name == "build" {
                        fig_util::open_url(format!(
                            "https://app.circleci.com/pipelines/github/withfig/macos/{pipeline_number}/workflows/{}",
                            workflow.id
                        ))?;
                        success = true;
                        break 'wait;
                    }
                }
            } else {
                let resp: CircleError = resp.json().await?;
                println!("error getting workflows: {}", resp.message);
                break 'wait;
            }

            interval.tick().await;
        }

        if !success {
            fig_util::open_url(format!(
                "https://app.circleci.com/pipelines/github/withfig/macos/{pipeline_number}",
            ))?;
        }
    } else {
        let resp: CircleError = resp.json().await?;
        eyre::bail!("CircleCI Error: {}", resp.message);
    }

    Ok(())
}
