use std::process::Command;
use std::str::FromStr;

use semver::{
    Prerelease,
    Version,
};
use serde::{
    Deserialize,
    Serialize,
};
use strum::{
    Display,
    EnumString,
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
#[derive(Deserialize, Serialize, EnumString, Display, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
#[strum(serialize_all = "snake_case")]
pub enum Channel {
    Nightly,
    Qa,
    Beta,
    Stable,
    None,
}

pub fn gen_nightly() -> String {
    let now = OffsetDateTime::now_utc().to_offset(offset!(-7));
    now.format(&format_description!("[year][month][day]")).unwrap()
}

pub fn run(args: &[&str]) -> eyre::Result<()> {
    run_wet(args, false)
}

pub fn run_wet(args: &[&str], dry: bool) -> eyre::Result<()> {
    if dry {
        print!("~");
    } else {
        print!("$");
    }
    print!(" {} ", args[0]);
    for arg in &args[1..] {
        print!("{arg} ");
    }
    println!();
    if !dry {
        let status = Command::new(args[0]).args(&args[1..]).status()?;
        if !status.success() {
            if let Some(code) = status.code() {
                eyre::bail!("Failed running command {}: exit code {code}", args.join(" "));
            } else {
                eyre::bail!("Failed running command {}", args.join(" "));
            }
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

fn read_manifest() -> Document {
    std::fs::read_to_string("Cargo.toml")
        .expect("failed reading manifest")
        .parse()
        .expect("failed parsing manifest")
}

pub fn read_version() -> Version {
    Version::parse(read_manifest()["workspace"]["package"]["version"].as_str().unwrap())
        .expect("failed parsing manifest")
}

pub fn read_channel() -> Channel {
    Channel::from_str(read_manifest()["workspace"]["metadata"]["channel"].as_str().unwrap())
        .expect("failed parsing manifest")
}

fn modify_manifest(f: impl FnOnce(&mut Document)) {
    let mut manifest = read_manifest();
    f(&mut manifest);
    std::fs::write("Cargo.toml", manifest.to_string()).expect("failed writing manifest");
}

pub fn write_version(version: &Version) {
    modify_manifest(|manifest| {
        manifest["workspace"]["package"]["version"] = value(version.to_string());
    });
}

pub fn write_channel(channel: &Channel) {
    modify_manifest(|manifest| {
        manifest["workspace"]["metadata"]["channel"] = value(channel.to_string());
    });
}

pub fn update_lockfile() -> eyre::Result<()> {
    run(&["cargo", "update", "--workspace"])
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
