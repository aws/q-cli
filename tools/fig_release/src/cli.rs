use std::path::PathBuf;

use clap::{
    Parser,
    Subcommand,
    ValueEnum,
};
use serde::Serialize;

#[derive(Parser, Debug)]
pub struct Cli {
    #[clap(subcommand)]
    pub subcommand: Sub,
    #[arg(short, long)]
    pub dry: bool,
    #[arg(short, long)]
    pub yes: bool,
}

#[derive(Subcommand, Debug)]
pub enum Sub {
    Changelog {
        #[clap(subcommand)]
        action: ChangelogAction,
    },
    Cut {
        #[arg(value_enum)]
        channel: Cut,
    },
    Package {
        path: PathBuf,
        #[arg(long, short, value_enum)]
        kind: PackageKind,
        #[arg(long, short, value_enum)]
        architecture: PackageArchitecture,
        #[arg(long, short, value_enum)]
        variant: PackageVariant,
    },
    Promote {
        #[arg(value_enum)]
        channel: Promote,
    },
    Bump {
        #[command(subcommand)]
        to: Option<BumpTo>,
    },
    Publish {
        build_targets: Vec<String>,
    },
    Debug {
        #[clap(subcommand)]
        action: DebugAction,
    },
}

#[derive(ValueEnum, Clone, Debug, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum PackageKind {
    WindowsInstaller,
    WindowsBundle,
    Dmg,
    Tar,
    Deb,
    Rpm,
}

#[derive(ValueEnum, Clone, Debug, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum PackageVariant {
    Online,
    Offline,
    Full,
    Headless,
}

#[derive(ValueEnum, Clone, Debug, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum PackageArchitecture {
    #[value(name = "x86_64")]
    #[serde(rename = "x86_64")]
    X86_64,
    #[value(name = "aarch64")]
    #[serde(rename = "aarch64")]
    AArch64,
    Universal,
}

#[derive(ValueEnum, Clone, Debug)]
pub enum Cut {
    Nightly,
    Release,
}

#[derive(ValueEnum, Clone, Debug)]
pub enum Promote {
    Beta,
    Stable,
}

#[derive(Subcommand, Debug)]
pub enum ChangelogAction {
    Edit,
    Add,
    Remove,
}

#[derive(Subcommand, Debug)]
pub enum DebugAction {
    GetIndexDirty,
    SetIndexDirty,
    GetSyncDirty,
    SetSyncDirty,
    ReadFile {
        path: PathBuf,
        #[arg(short, long)]
        base: bool,
    },
    UpdateLockfile,
}

#[derive(Subcommand, Debug)]
pub enum BumpTo {
    Qa,
    Beta,
}
