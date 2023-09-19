use clap::Args;
use eyre::Result;
use fig_sync::dotfiles::api::DotfileData;

#[derive(Debug, Args, PartialEq, Eq)]
pub struct ExportArgs {}

impl ExportArgs {
    pub async fn execute(&self) -> Result<()> {
        println!("Exporting...");

        let current_unix = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .expect("Time went backwards");
        let export_dir = std::env::current_dir()?.join(format!("fig-export-{}", current_unix.as_secs()));

        std::fs::create_dir(&export_dir)?;

        // dotfiles
        let dotfiles_dir = export_dir.join("dotfiles");
        std::fs::create_dir(&dotfiles_dir)?;
        for shell in fig_util::Shell::all() {
            let data_path = shell.get_data_path()?;
            let get_dotfile_source = || {
                let raw = std::fs::read_to_string(data_path).ok()?;
                let source: DotfileData = serde_json::from_str(&raw).ok()?;
                Some(source.dotfile)
            };

            if let Some(dotfile_source) = get_dotfile_source() {
                let dotfile_path = export_dir.join("dotfiles").join(match shell {
                    fig_util::Shell::Bash => "dotfile.bash",
                    fig_util::Shell::Zsh => "dotfile.zsh",
                    fig_util::Shell::Fish => "dotfile.fish",
                    fig_util::Shell::Nu => "dotfile.nu",
                });

                std::fs::write(dotfile_path, dotfile_source)?;
            }
        }

        let access_dir = export_dir.join("access");
        std::fs::create_dir(&access_dir)?;

        let mut hosts = vec![];
        let mut identities = vec![];

        let teams = fig_api_client::user::teams().await?;

        let mut host_tasks = vec![];
        let mut identity_tasks = vec![];

        host_tasks.push(tokio::spawn(fig_api_client::access::hosts(None)));
        identity_tasks.push(tokio::spawn(fig_api_client::access::identities(None)));

        for team in teams {
            host_tasks.push(tokio::spawn(fig_api_client::access::hosts(Some(team.name.clone()))));
            identity_tasks.push(tokio::spawn(fig_api_client::access::identities(Some(team.name))));
        }

        for task in host_tasks {
            hosts.extend(task.await??);
        }
        for task in identity_tasks {
            identities.extend(task.await??);
        }

        serde_json::to_writer_pretty(
            std::fs::File::create(export_dir.join("access").join("hosts.json"))?,
            &hosts,
        )?;

        serde_json::to_writer_pretty(
            std::fs::File::create(export_dir.join("access").join("identities.json"))?,
            &identities,
        )?;

        let scripts_dir = export_dir.join("scripts");
        std::fs::create_dir(&scripts_dir)?;

        let scripts = fig_api_client::scripts::scripts().await?;

        for script in scripts {
            let script_path = scripts_dir.join(format!("{}.{}.json", script.namespace, script.name));
            serde_json::to_writer_pretty(std::fs::File::create(script_path)?, &script)?;
        }

        let home_dir = fig_util::directories::home_dir()?;
        let user_readable_dir = match export_dir.strip_prefix(home_dir) {
            Ok(user_readable_dir) => format!("~/{}", user_readable_dir.display()),
            Err(_) => export_dir.display().to_string(),
        };

        println!("Exported to: {user_readable_dir}");

        Ok(())
    }
}
