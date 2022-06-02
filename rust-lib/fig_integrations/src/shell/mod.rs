use std::fs::File;
use std::io::Write;
use std::path::{
    Path,
    PathBuf,
};

use anyhow::{
    Context,
    Result,
};
use clap::ArgEnum;
use fig_util::Shell;
use regex::{
    Regex,
    RegexSet,
};
use serde::{
    Deserialize,
    Serialize,
};

use crate::{
    backup_file,
    FileIntegration,
    InstallationError,
    Integration,
};

#[derive(Debug, Copy, Clone, PartialEq, Eq, ArgEnum, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum When {
    Pre,
    Post,
}

impl std::fmt::Display for When {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            When::Pre => write!(f, "pre"),
            When::Post => write!(f, "post"),
        }
    }
}

pub trait ShellExt {
    fn get_shell_integrations(&self) -> Result<Vec<Box<dyn ShellIntegration>>>;
    fn get_fig_integration_source(&self, when: &When) -> &'static str;
}

impl ShellExt for Shell {
    fn get_shell_integrations(&self) -> Result<Vec<Box<dyn ShellIntegration>>> {
        let config_dir = self.get_config_directory().context("Failed to get base directories")?;

        let integrations: Vec<Box<dyn ShellIntegration>> = match self {
            Shell::Bash => {
                let mut configs = vec![".bashrc"];
                let other_configs = [".profile", ".bash_login", ".bash_profile"];

                configs.extend(other_configs.into_iter().filter(|f| config_dir.join(f).exists()));

                // Include .profile if none of [.profile, .bash_login, .bash_profile] exist.
                if configs.len() == 1 {
                    configs.push(other_configs[0]);
                }

                configs
                    .into_iter()
                    .map(|filename| {
                        Box::new(DotfileShellIntegration {
                            pre: true,
                            post: true,
                            shell: *self,
                            dotfile_directory: config_dir.clone(),
                            dotfile_name: filename,
                        }) as Box<dyn ShellIntegration>
                    })
                    .collect()
            },
            Shell::Zsh => vec![".zshrc", ".zprofile"]
                .into_iter()
                .map(|filename| {
                    Box::new(DotfileShellIntegration {
                        pre: true,
                        post: true,
                        shell: *self,
                        dotfile_directory: config_dir.clone(),
                        dotfile_name: filename,
                    }) as Box<dyn ShellIntegration>
                })
                .collect(),
            Shell::Fish => {
                let fish_config_dir = config_dir.join("conf.d");
                vec![
                    Box::new(ShellScriptShellIntegration {
                        when: When::Pre,
                        shell: *self,
                        path: fish_config_dir.join("00_fig_pre.fish"),
                    }),
                    Box::new(ShellScriptShellIntegration {
                        when: When::Post,
                        shell: *self,
                        path: fish_config_dir.join("99_fig_post.fish"),
                    }),
                ]
            },
        };

        Ok(integrations)
    }

    fn get_fig_integration_source(&self, when: &When) -> &'static str {
        match (self, when) {
            (Shell::Fish, When::Pre) => include_str!("scripts/pre.fish"),
            (Shell::Fish, When::Post) => include_str!("scripts/post.fish"),
            (Shell::Zsh, When::Pre) => include_str!("scripts/pre.sh"),
            (Shell::Zsh, When::Post) => include_str!("scripts/post.zsh"),
            (Shell::Bash, When::Pre) => {
                concat!(
                    "function __fig_source_bash_preexec() {\n",
                    include_str!("scripts/bash-preexec.sh"),
                    "}\n",
                    "__fig_source_bash_preexec\n",
                    "function __bp_adjust_histcontrol() { :; }\n",
                    include_str!("scripts/pre.sh")
                )
            },
            (Shell::Bash, When::Post) => {
                concat!(
                    "function __fig_source_bash_preexec() {\n",
                    include_str!("scripts/bash-preexec.sh"),
                    "}\n",
                    "__fig_source_bash_preexec\n",
                    "function __bp_adjust_histcontrol() { :; }\n",
                    include_str!("scripts/post.bash")
                )
            },
        }
    }
}

pub trait ShellIntegration: Send + Sync + Integration + ShellIntegrationClone {
    // The unique name of the integration file
    fn file_name(&self) -> &str;
    fn get_shell(&self) -> Shell;
    fn path(&self) -> PathBuf;
}

impl std::fmt::Display for dyn ShellIntegration {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} ({})", self.get_shell(), self.path().display())
    }
}

pub trait ShellIntegrationClone {
    fn clone_box(&self) -> Box<dyn ShellIntegration>;
}

impl<T> ShellIntegrationClone for T
where
    T: 'static + ShellIntegration + Clone,
{
    fn clone_box(&self) -> Box<dyn ShellIntegration> {
        Box::new(self.clone())
    }
}

// We can now implement Clone manually by forwarding to clone_box.
impl Clone for Box<dyn ShellIntegration> {
    fn clone(&self) -> Box<dyn ShellIntegration> {
        self.clone_box()
    }
}

#[derive(Debug, Clone)]
pub struct ShellScriptShellIntegration {
    pub shell: Shell,
    pub when: When,
    pub path: PathBuf,
}

fn get_prefix(s: &str) -> &str {
    match s.find('.') {
        Some(i) => &s[..i],
        None => s,
    }
}

impl ShellScriptShellIntegration {
    fn get_file_integration(&self) -> FileIntegration {
        FileIntegration {
            path: self.path.clone(),
            contents: self.get_contents(),
        }
    }

    fn get_name(&self) -> Option<&str> {
        self.path.file_name().and_then(|s| s.to_str())
    }

    fn get_contents(&self) -> String {
        let Self { shell, when, path } = self;
        let rcfile = match path.file_name().and_then(|x| x.to_str()) {
            Some(name) => format!(" --rcfile {}", get_prefix(name)),
            None => "".into(),
        };
        match self.shell {
            Shell::Fish => format!("eval (fig init {shell} {when}{rcfile} | string split0)"),
            _ => format!("eval \"$(fig init {shell} {when}{rcfile})\""),
        }
    }
}

impl Integration for ShellScriptShellIntegration {
    fn is_installed(&self) -> Result<(), InstallationError> {
        self.get_file_integration().is_installed()
    }

    fn install(&self, backup_dir: Option<&Path>) -> Result<()> {
        self.get_file_integration().install(backup_dir)
    }

    fn uninstall(&self) -> Result<()> {
        self.get_file_integration().uninstall()
    }
}

impl ShellIntegration for ShellScriptShellIntegration {
    fn file_name(&self) -> &str {
        self.get_name().unwrap_or("unknown_script")
    }

    fn get_shell(&self) -> Shell {
        self.shell
    }

    fn path(&self) -> PathBuf {
        self.path.clone()
    }
}

// zsh and bash integration where we modify a dotfile with pre/post hooks that reference
// script files.
#[derive(Debug, Clone)]
pub struct DotfileShellIntegration {
    pub shell: Shell,
    pub pre: bool,
    pub post: bool,
    pub dotfile_directory: PathBuf,
    pub dotfile_name: &'static str,
}

impl DotfileShellIntegration {
    fn dotfile_path(&self) -> PathBuf {
        self.dotfile_directory.join(self.dotfile_name)
    }

    fn script_integration(&self, when: When) -> Result<ShellScriptShellIntegration> {
        let integration_file_name = format!(
            "{}.{}.{}",
            Regex::new(r"^\.").unwrap().replace_all(self.dotfile_name, ""),
            when,
            self.shell
        );
        Ok(ShellScriptShellIntegration {
            shell: self.shell,
            when,
            path: fig_directories::fig_dir()
                .context("Could not get fig dir")?
                .join("shell")
                .join(integration_file_name),
        })
    }

    fn description(&self, when: When) -> String {
        match when {
            When::Pre => "# Fig pre block. Keep at the top of this file.".into(),
            When::Post => "# Fig post block. Keep at the bottom of this file.".into(),
        }
    }

    fn legacy_regexes(&self, when: When) -> Result<RegexSet> {
        let shell = self.shell;

        let eval_line = match shell {
            Shell::Fish => format!("eval (fig init {shell} {when} | string split0)"),
            _ => format!("eval \"$(fig init {shell} {when})\""),
        };

        let old_eval_source = match when {
            When::Pre => match self.shell {
                Shell::Fish => format!("set -Ua fish_user_paths $HOME/.local/bin\n{eval_line}"),
                _ => format!("export PATH=\"${{PATH}}:${{HOME}}/.local/bin\"\n{eval_line}"),
            },
            When::Post => eval_line,
        };

        let old_file_regex = match when {
            When::Pre => r"\[ -s ~/\.fig/shell/pre\.sh \] && source ~/\.fig/shell/pre\.sh\n?",
            When::Post => r"\[ -s ~/\.fig/fig\.sh \] && source ~/\.fig/fig\.sh\n?",
        };
        let old_eval_regex = format!(
            r#"(?m)(?:{}\n)?^{}\n{{0,2}}"#,
            regex::escape(&self.description(when)),
            regex::escape(&old_eval_source),
        );
        let old_source_regex = format!(
            r#"(?m)(?:{}\n)?^{}\n{{0,2}}"#,
            regex::escape(&self.description(when)),
            regex::escape(&self.legacy_source_text(when)?),
        );

        Ok(RegexSet::new(&[old_file_regex, &old_eval_regex, &old_source_regex])?)
    }

    fn legacy_source_text(&self, when: When) -> Result<String> {
        let home = fig_directories::home_dir().context("Could not get home dir")?;
        let integration_path = self.script_integration(when)?.path;
        let path = integration_path.strip_prefix(home)?;
        Ok(format!(". \"$HOME/{}\"", path.display()))
    }

    fn source_text(&self, when: When) -> Result<String> {
        let home = fig_directories::home_dir().context("Could not get home dir")?;
        let integration_path = self.script_integration(when)?.path;
        let path = format!("\"$HOME/{}\"", integration_path.strip_prefix(home)?.display());

        match self.shell {
            Shell::Fish => Ok(format!("if test -f {path}; . {path}; end")),
            _ => Ok(format!("[[ -f {path} ]] && . {path}")),
        }
    }

    fn source_regex(&self, when: When, constrain_position: bool) -> Result<Regex> {
        let regex = format!(
            r#"{}(?:{}\n)?{}\n{{0,2}}{}"#,
            if constrain_position && when == When::Pre {
                "^"
            } else {
                ""
            },
            regex::escape(&self.description(when)),
            regex::escape(&self.source_text(when)?),
            if constrain_position && when == When::Post {
                "$"
            } else {
                ""
            },
        );
        Regex::new(&regex).context("Invalid source regex")
    }

    fn remove_from_text(&self, text: impl Into<String>, when: When) -> Result<String> {
        let source_regex = self.source_regex(when, false)?;
        let mut regexes = vec![source_regex];
        regexes.extend(
            self.legacy_regexes(when)?
                .patterns()
                .iter()
                .map(|r| Regex::new(r).unwrap()),
        );
        Ok(regexes
            .iter()
            .fold::<String, _>(text.into(), |acc, reg| reg.replace_all(&acc, "").into()))
    }

    fn matches_text(&self, text: &str, when: When) -> Result<(), InstallationError> {
        let dotfile = self.dotfile_path();
        if self.legacy_regexes(when)?.is_match(text) {
            let message = format!("{} has legacy {} integration.", dotfile.display(), when);
            return Err(InstallationError::LegacyInstallation(message.into()));
        }
        if !self.source_regex(when, false)?.is_match(text) {
            let message = format!("{} does not source {} integration", dotfile.display(), when);
            return Err(InstallationError::NotInstalled(message.into()));
        }
        if !self.source_regex(when, true)?.is_match(text) {
            let position = match when {
                When::Pre => "first",
                When::Post => "last",
            };
            let message = format!(
                "{} does not source {} integration {}",
                dotfile.display(),
                when,
                position
            );
            return Err(InstallationError::ImproperInstallation(message.into()));
        }
        Ok(())
    }
}

impl Integration for DotfileShellIntegration {
    fn install(&self, backup_dir: Option<&Path>) -> Result<()> {
        if self.is_installed().is_ok() {
            return Ok(());
        }

        let dotfile = self.dotfile_path();
        let mut contents = if dotfile.exists() {
            backup_file(&dotfile, backup_dir)?;
            self.uninstall()?;
            std::fs::read_to_string(&dotfile)?
        } else {
            String::new()
        };

        let original_contents = contents.clone();

        if self.pre {
            self.script_integration(When::Pre)?.install(backup_dir)?;
            contents = format!(
                "{}\n{}\n{}",
                self.description(When::Pre),
                self.source_text(When::Pre)?,
                contents,
            );
        }

        if self.post {
            self.script_integration(When::Post)?.install(backup_dir)?;
            contents = format!(
                "{}\n{}\n{}\n",
                contents,
                self.description(When::Post),
                self.source_text(When::Post)?,
            );
        }

        if contents.ne(&original_contents) {
            let mut file = File::create(&dotfile)?;
            file.write_all(contents.as_bytes())?;
        }

        Ok(())
    }

    fn is_installed(&self) -> Result<(), InstallationError> {
        let dotfile = self.dotfile_path();
        let filtered_contents: String = match std::fs::read_to_string(&dotfile) {
            // Remove comments and empty lines.
            Ok(contents) => Regex::new(r"^\s*(#.*)?\n").unwrap().replace_all(&contents, "").into(),
            _ => {
                return Err(InstallationError::FileDoesNotExist(dotfile.into()));
            },
        };

        if self.pre {
            self.matches_text(&filtered_contents, When::Pre)?;
            self.script_integration(When::Pre)?.is_installed()?;
        }

        if self.post {
            self.matches_text(&filtered_contents, When::Post)?;
            self.script_integration(When::Post)?.is_installed()?;
        }

        Ok(())
    }

    fn uninstall(&self) -> Result<()> {
        let dotfile = self.dotfile_path();
        if dotfile.exists() {
            let mut contents = std::fs::read_to_string(&dotfile)?;

            // Remove comment lines
            contents = Regex::new(r"(?mi)^#.*fig.*var.*$\n?")?
                .replace_all(&contents, "")
                .into();

            contents = Regex::new(r"(?mi)^#.*Please make sure this block is at the .* of this file.*$\n?")?
                .replace_all(&contents, "")
                .into();

            if self.pre {
                contents = self.remove_from_text(&contents, When::Pre)?;
            }

            if self.post {
                contents = self.remove_from_text(&contents, When::Post)?;
            }

            contents = contents.trim().to_string();
            contents.push('\n');

            std::fs::write(&dotfile, contents.as_bytes())?;
        }

        if self.pre {
            self.script_integration(When::Pre)?.uninstall()?;
        }

        if self.post {
            self.script_integration(When::Post)?.uninstall()?;
        }

        Ok(())
    }
}

impl ShellIntegration for DotfileShellIntegration {
    fn get_shell(&self) -> Shell {
        self.shell
    }

    fn path(&self) -> PathBuf {
        self.dotfile_path()
    }

    fn file_name(&self) -> &str {
        self.dotfile_name
    }
}

#[cfg(test)]
mod test {
    use std::io::Write;
    use std::process::{
        Command,
        Stdio,
    };

    use super::*;

    fn check_script(shell: Shell, when: When) {
        let shell_arg = format!("--shell=bash");
        let mut child = Command::new("shellcheck")
            .args(&[&shell_arg, "--color=always", "-"])
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .unwrap();

        let mut stdin = child.stdin.take().unwrap();
        std::thread::spawn(move || {
            stdin
                .write_all(shell.get_fig_integration_source(&when).as_bytes())
                .unwrap();
        });

        let output = child.wait_with_output().unwrap();
        if !output.status.success() {
            let stdout = String::from_utf8(output.stdout).unwrap();
            let stderr = String::from_utf8(output.stderr).unwrap();

            if !stdout.is_empty() {
                println!("{stdout}");
            }

            if !stderr.is_empty() {
                eprintln!("{stderr}");
            }

            panic!();
        }
    }

    #[test]
    fn shellcheck_bash_pre() {
        check_script(Shell::Bash, When::Pre);
    }

    #[test]
    fn shellcheck_bash_post() {
        check_script(Shell::Bash, When::Post);
    }

    #[test]
    fn shellcheck_zsh_pre() {
        check_script(Shell::Bash, When::Pre);
    }

    #[test]
    fn shellcheck_zsh_post() {
        check_script(Shell::Zsh, When::Post);
    }
}
