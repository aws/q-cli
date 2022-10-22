use std::fs::OpenOptions;
use std::process::Command;

use semver::Prerelease;
use serde::{
    Deserialize,
    Serialize,
};
use time::macros::{
    format_description,
    offset,
};
use time::OffsetDateTime;
use toml_edit::{
    value,
    Document,
};

#[derive(Deserialize, Serialize)]
pub struct ReleaseFile {
    pub version: String,
    pub channel: Option<Channel>,
    pub changelog: Vec<String>,
}

#[derive(Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Channel {
    Nightly,
    Qa,
    Beta,
    Stable,
}

pub fn read_release_file() -> eyre::Result<ReleaseFile> {
    Ok(serde_yaml::from_reader(
        OpenOptions::new().read(true).open("release.yaml")?,
    )?)
}

pub fn write_release_file(release: &ReleaseFile) -> eyre::Result<()> {
    serde_yaml::to_writer(
        OpenOptions::new().write(true).truncate(true).open("release.yaml")?,
        release,
    )?;
    Ok(())
}

pub fn gen_nightly() -> String {
    let now = OffsetDateTime::now_utc().to_offset(offset!(-7));
    now.format(&format_description!("[year][month][day]")).unwrap()
}

pub fn run(args: &[&str]) -> eyre::Result<()> {
    print!("$ {} ", args[0]);
    for arg in &args[1..] {
        print!("{arg} ");
    }
    println!();
    let status = Command::new(args[0]).args(&args[1..]).status()?;
    if !status.success() {
        if let Some(code) = status.code() {
            eyre::bail!("Failed running command {}: exit code {code}", args.join(" "));
        } else {
            eyre::bail!("Failed running command {}", args.join(" "));
        }
    }
    Ok(())
}

pub fn run_stdout(args: &[&str]) -> eyre::Result<String> {
    print!("$ {} ", args[0]);
    for arg in &args[1..] {
        print!("{arg} ");
    }
    println!();
    let output = Command::new(args[0]).args(&args[1..]).output()?;
    if !output.status.success() {
        if let Some(code) = output.status.code() {
            eyre::bail!("Failed running command {}: exit code {code}", args.join(" "));
        } else {
            eyre::bail!("Failed running command {}", args.join(" "));
        }
    }
    Ok(String::from_utf8_lossy(&output.stdout).to_string())
}

pub fn extract_number(pre: &Prerelease) -> eyre::Result<u64> {
    Ok(pre
        .as_str()
        .chars()
        .filter(|x| x.is_numeric())
        .collect::<String>()
        .parse()?)
}

pub fn sync_version(release: &ReleaseFile) -> eyre::Result<()> {
    let cargo_toml = std::fs::read_to_string("Cargo.toml")?;
    let mut document = cargo_toml.parse::<Document>()?;

    document["workspace"]["package"]["version"] = value(release.version.to_string());

    std::fs::write("Cargo.toml", document.to_string())?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use semver::Prerelease;

    use super::{
        extract_number,
        gen_nightly,
    };

    #[test]
    fn test_gen_nightly() {
        assert_eq!(gen_nightly().len(), 8);
    }

    #[test]
    fn test_extract_number() {
        assert_eq!(extract_number(&Prerelease::new("beta.3").unwrap()).unwrap(), 3);
        assert_eq!(extract_number(&Prerelease::new("alpha.24").unwrap()).unwrap(), 24);
    }
}
