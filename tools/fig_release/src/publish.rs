use std::time::Duration;

use serde::{
    Deserialize,
    Serialize,
};

use crate::utils::{
    read_channel,
    read_version,
    run_stdout,
    run_wet,
    Channel,
};

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
    publish: bool,
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

pub async fn publish(build_targets: Vec<String>, dry: bool, yes: bool) -> eyre::Result<()> {
    if build_targets.is_empty() {
        eyre::bail!("Didn't specify any build targets");
    }

    let channel = read_channel();
    let publish = match channel {
        Channel::None => {
            eprintln!("No channel specified, this will not be published");
            false
        },
        _ => true,
    };

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

    if !yes {
        let version = read_version();

        if !dialoguer::Confirm::new()
            .with_prompt(&format!(
                "Are you sure you want to deploy {version} on branch {current_branch} ({commit_hash}) to channel {channel}"
            ))
            .interact()?
        {
            eyre::bail!("Cancelled");
        }
    }

    let resp = client
        .post("https://circleci.com/api/v2/project/github/withfig/macos/pipeline")
        .header("Circle-Token", &token)
        .json(&TriggerPipelineRequest {
            branch: current_branch.clone(),
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
                publish,
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

        if success {
            let url = format!("https://app.circleci.com/pipelines/github/withfig/macos/{pipeline_number}");
            println!("---> {url} <---");
            fig_util::open_url(url)?;
        }
    } else {
        let resp: CircleError = resp.json().await?;
        eyre::bail!("CircleCI Error: {}", resp.message);
    }

    // Trigger gh action to update the cli spec
    if channel == Channel::Stable {
        println!("Triggering gh action to update the cli spec...");

        run_wet(
            &[
                "gh",
                "workflow",
                "run",
                "update-fig-cli-spec.yaml",
                "--repo",
                "withfig/macos",
                "--ref",
                current_branch.as_str(),
            ],
            dry,
        )?;
    }

    Ok(())
}
