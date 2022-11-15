use std::borrow::Cow;
use std::env;
use std::fmt::Display;
use std::io::{
    stdin,
    stdout,
    Write,
};

use clap::Args;
use crossterm::tty::IsTty;
use eyre::Result;
use fig_integrations::shell::{
    ShellExt,
    When,
};
use fig_request::auth::is_logged_in;
use fig_sync::dotfiles::api::DotfileData;
use fig_util::{
    Shell,
    Terminal,
};

use crate::util::app_path_from_bundle_id;

#[derive(Debug, Args, PartialEq, Eq)]
pub struct InitArgs {
    /// The shell to generate the dotfiles for
    #[arg(value_enum)]
    shell: Shell,
    /// When to generate the dotfiles for
    #[arg(value_enum)]
    when: When,
    #[arg(long)]
    rcfile: Option<String>,
    /// Whether to skip loading dotfiles
    #[arg(long)]
    skip_dotfiles: bool,
}

impl InitArgs {
    pub async fn execute(&self) -> Result<()> {
        let InitArgs {
            shell,
            when,
            rcfile,
            skip_dotfiles,
        } = self;
        match shell_init(shell, when, rcfile, *skip_dotfiles) {
            Ok(source) => writeln!(stdout(), "{source}"),
            Err(err) => writeln!(stdout(), "# Could not load source: {err}"),
        }
        .ok();
        Ok(())
    }
}

#[derive(PartialEq, Eq)]
enum GuardAssignment {
    BeforeSourcing,
    AfterSourcing,
}

fn assign_shell_variable(shell: &Shell, name: impl Display, exported: bool) -> String {
    match (shell, exported) {
        (Shell::Bash | Shell::Zsh, false) => format!("{name}=1"),
        (Shell::Bash | Shell::Zsh, true) => format!("export {name}=1"),
        (Shell::Fish, false) => format!("set -g {name} 1"),
        (Shell::Fish, true) => format!("set -gx {name} 1"),
    }
}

fn guard_source(
    shell: &Shell,
    export: bool,
    guard_var: impl Display,
    assignment: GuardAssignment,
    source: impl Into<Cow<'static, str>>,
) -> String {
    let mut output: Vec<Cow<'static, str>> = Vec::with_capacity(4);

    output.push(match shell {
        Shell::Bash | Shell::Zsh => format!("if [ -z \"${{{guard_var}}}\" ]; then").into(),
        Shell::Fish => format!("if test -z \"${guard_var}\"").into(),
    });

    let shell_var = assign_shell_variable(shell, guard_var, export);
    match assignment {
        GuardAssignment::BeforeSourcing => {
            // If script may trigger rc file to be rerun, guard assignment must happen first to avoid recursion
            output.push(format!("  {shell_var}").into());
            for line in source.into().lines() {
                output.push(format!("  {line}").into());
            }
        },
        GuardAssignment::AfterSourcing => {
            for line in source.into().lines() {
                output.push(format!("  {line}").into());
            }
            output.push(format!("  {shell_var}").into());
        },
    }

    output.push(
        match shell {
            Shell::Bash | Shell::Zsh => "fi\n",
            Shell::Fish => "end\n",
        }
        .into(),
    );

    output.join("\n")
}

fn shell_init(shell: &Shell, when: &When, rcfile: &Option<String>, skip_dotfiles: bool) -> Result<String> {
    // Do not print any shell integrations for `.profile` as it can cause issues on launch
    if std::env::consts::OS == "linux" && matches!(rcfile.as_deref(), Some("profile")) {
        return Ok("".into());
    }

    if !fig_settings::state::get_bool_or("shell-integrations.enabled", true) {
        return Ok(guard_source(
            shell,
            false,
            "FIG_SHELL_INTEGRATION_DISABLED",
            GuardAssignment::AfterSourcing,
            "echo '[Debug]: fig shell integration is disabled.'",
        ));
    }

    let mut to_source = Vec::new();

    if let When::Post = when {
        if !matches!(
            (shell, rcfile.as_deref()),
            (Shell::Zsh, Some("zprofile")) | (Shell::Bash, Some("profile") | Some("bash_profile"))
        ) && fig_settings::state::get_bool_or("dotfiles.enabled", true)
            && !skip_dotfiles
        {
            // Add dotfiles sourcing
            let data_path = shell.get_data_path()?;
            let get_dotfile_source = || {
                let raw = std::fs::read_to_string(data_path).ok()?;
                let source: DotfileData = serde_json::from_str(&raw).ok()?;
                Some(source.dotfile)
            };

            if let Some(source) = get_dotfile_source() {
                to_source.push(guard_source(
                    shell,
                    false,
                    "FIG_DOTFILES_SOURCED",
                    GuardAssignment::AfterSourcing,
                    source,
                ));
            }
        }

        if stdin().is_tty() && env::var_os("PROCESS_LAUNCHED_BY_FIG").is_none() {
            // if no value, assume that we have seen onboarding already.
            // this is explicitly set in onboarding in macOS app.
            let has_seen_onboarding: bool = fig_settings::state::get_bool_or("user.onboarding", true);

            // cfg_if! {
            //     if #[cfg(target_os = "macos")] {
            //         let terminal_supports_onboarding = {
            //             let terminal = Terminal::parent_terminal();
            //             [Some(Terminal::Iterm), Some(Terminal::TerminalApp)].contains(&terminal)
            //         };
            //     } else {
            //         let terminal_supports_onboarding = true;
            //     }
            // }

            if is_logged_in() && !has_seen_onboarding {
                to_source.push(match shell {
                    Shell::Bash | Shell::Zsh => "(fig restart daemon &> /dev/null &)".into(),
                    Shell::Fish => "begin; fig restart daemon &> /dev/null &; end".into(),
                });

                to_source.push("fig app onboarding".into())
            }
        }

        if fig_settings::state::get_bool_or("shell-integrations.immediateLogin", false)
            && fig_settings::state::set_value("shell-integrations.immediateLogin", false).is_ok()
        {
            to_source.push("fig login --not-now".into());
        }
    }

    let is_jetbrains_terminal = Terminal::is_jetbrains_terminal();

    if when == &When::Pre && shell == &Shell::Bash && is_jetbrains_terminal {
        // JediTerm does not launch as a 'true' login shell, so our normal "shopt -q login_shell" check does
        // not work. Thus, FIG_IS_LOGIN_SHELL will be incorrect. We must manually set it so the
        // user's bash_profile is sourced. https://github.com/JetBrains/intellij-community/blob/master/plugins/terminal/resources/jediterm-bash.in
        to_source.push("FIG_IS_LOGIN_SHELL=1".into())
    }

    let shell_integration_source = shell.get_fig_integration_source(when);
    to_source.push(shell_integration_source.into());

    if when == &When::Pre && is_jetbrains_terminal {
        // Manually call JetBrains shell integration after exec-ing to figterm.
        // This may recursively call out to bashrc/zshrc so make sure to assign guard variable first.

        let get_jetbrains_source = if let Some(bundle_id) = std::env::var_os("__CFBundleIdentifier") {
            if let Some(bundle) = app_path_from_bundle_id(bundle_id) {
                // The source for JetBrains shell integrations can be found here.
                // https://github.com/JetBrains/intellij-community/tree/master/plugins/terminal/resources
                match shell {
                    Shell::Bash => Some(format!("source '{bundle}/Contents/plugins/terminal/jediterm-bash.in'",)),
                    Shell::Zsh => Some(format!("source '{bundle}/Contents/plugins/terminal/.zshenv'",)),
                    Shell::Fish => Some(format!("source '{bundle}/Contents/plugins/terminal/fish/config.fish'",)),
                }
            } else {
                None
            }
        } else {
            None
        };

        if let Some(source) = get_jetbrains_source {
            to_source.push(guard_source(
                shell,
                false,
                "FIG_JETBRAINS_SHELL_INTEGRATION",
                GuardAssignment::BeforeSourcing,
                source,
            ));
        }
    }

    #[cfg(target_os = "macos")]
    if when == &When::Post
        && !fig_integrations::input_method::InputMethod::default()
            .is_enabled()
            .unwrap_or(false)
    {
        use crossterm::style::Stylize;

        if let Some(terminal) = Terminal::parent_terminal() {
            let prompt_state_key = format!("prompt.input-method.{}.count", terminal.internal_id());
            let prompt_count = fig_settings::state::get_int_or(&prompt_state_key, 0);

            if terminal.is_input_dependant() && prompt_count < 2 {
                fig_settings::state::set_value(&prompt_state_key, prompt_count + 1)?;

                to_source.push(guard_source(
                    shell,
                    false,
                    "FIG_INPUT_METHOD_PROMPT",
                    GuardAssignment::AfterSourcing,
                    format!(
                        "printf '\\n🚀 Fig now supports {terminal} Terminal!\\nEnable integrations with {terminal} by \
                         running:\\n  {}\\n\\n'\n",
                        "fig integrations install input-method".magenta()
                    ),
                ));
            }
        }
    }

    Ok(to_source.join("\n"))
}
