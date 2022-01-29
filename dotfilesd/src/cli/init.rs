use crate::util::shell::Shell;
use anyhow::Result;
use clap::ArgEnum;
use serde::{Deserialize, Serialize};

#[derive(Debug, Copy, Clone, PartialEq, Eq, ArgEnum)]
pub enum When {
    Pre,
    Post,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct DotfileData {
    dotfile: String,
}

fn shell_init(shell: &Shell, when: &When) -> Result<String> {
    let raw = std::fs::read_to_string(shell.get_cache_path()?)?;
    let source: DotfileData = serde_json::from_str(&raw)?;

    match when {
        When::Pre => Ok(String::new()),
        When::Post => Ok(source.dotfile),
    }
}

pub fn shell_init_cli(shell: &Shell, when: &When) -> Result<()> {
    println!("# {:?} for {:?}", when, shell);
    match shell_init(shell, when) {
        Ok(source) => println!("{}", source),
        Err(err) => println!("# Could not load source: {}", err),
    }

    Ok(())
}
