use std::borrow::Cow;
use std::env;
use std::fmt::Display;
use std::io::stdin;

use anyhow::{
    Context,
    Result,
};
use crossterm::style::Stylize;
use crossterm::tty::IsTty;
use fig_auth::is_logged_in;
use fig_install::dotfiles::api::DotfileData;
use fig_integrations::shell::{
    ShellExt,
    When,
};
use fig_util::{
    Shell,
    Terminal,
};

use crate::util::app_path_from_bundle_id;

#[derive(PartialEq, Eq)]
enum GuardAssignment {
    BeforeSourcing,
    AfterSourcing,
}

#[must_use]
fn assign_shell_variable(shell: &Shell, name: impl Display, exported: bool) -> String {
    match (shell, exported) {
        (Shell::Bash | Shell::Zsh, false) => format!("{name}=1"),
        (Shell::Bash | Shell::Zsh, true) => format!("export {name}=1"),
        (Shell::Fish, false) => format!("set -g {name} 1"),
        (Shell::Fish, true) => format!("set -gx {name} 1"),
    }
}

#[must_use]
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

fn shell_init(shell: &Shell, when: &When, rcfile: Option<String>) -> Result<String> {
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
        {
            // Add dotfiles sourcing
            let get_dotfile_source = || {
                let raw = std::fs::read_to_string(shell.get_data_path().context("Failed to get shell data path").ok()?)
                    .ok()?;
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
            // this is explictly set in onboarding in macOS app.
            let has_see_onboarding: bool = fig_settings::state::get_bool_or("user.onboarding", true);

            let terminal = Terminal::parent_terminal();

            if is_logged_in()
                && !has_see_onboarding
                && [Some(Terminal::Iterm), Some(Terminal::TerminalApp)].contains(&terminal)
            {
                to_source.push(match shell {
                    Shell::Bash | Shell::Zsh => "(fig restart daemon &> /dev/null &)".into(),
                    Shell::Fish => "begin; fig restart daemon &> /dev/null &; end".into(),
                });

                to_source.push("fig app onboarding".into())
            } else {
                // not showing onboarding
                to_source.push(guard_source(
                    shell,
                    false,
                    "FIG_CHECKED_PROMPTS",
                    GuardAssignment::AfterSourcing,
                    match shell {
                        Shell::Bash | Shell::Zsh => "(fig app prompts &)",
                        Shell::Fish => "begin; fig app prompts &; end",
                    },
                ));
            }
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

    if when == &When::Post
        && !fig_settings::state::get_bool_or("input-method.enabled", false)
        && !fig_settings::settings::get_bool_or("integrations.experimental", false)
    {
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
                        "fig install --input-method".magenta()
                    ),
                ));
            }
        }
    }

    // April Fools
    if fig_settings::settings::get_bool_or("command-not-found.beta", false) {
        let after_text = format!(
            "Command not found: {}\nTo disable Terminal Reactions™️ by Fig run: fig settings command-not-found.beta \
             false",
            match shell {
                Shell::Bash | Shell::Zsh => "$0",
                Shell::Fish => "$argv[1]",
            }
        );

        let fools_cmd = format!(
            "fig _ animation -f random --before-text \"Loading Terminal Reactions™️ by Fig...\" --after-text \
             \"{after_text}\"\nreturn 127"
        );

        to_source.push(guard_source(
            shell,
            false,
            "FIG_APRIL_FOOLS_GUARD",
            GuardAssignment::AfterSourcing,
            match shell {
                Shell::Bash => format!("command_not_found_handle() {{ {fools_cmd}; }}"),
                Shell::Zsh => format!("command_not_found_handler() {{ {fools_cmd}; }}"),
                Shell::Fish => format!("function fish_command_not_found\n    {fools_cmd}\nend"),
            },
        ));
    }

    Ok(to_source.join("\n"))
}

pub async fn shell_init_cli(shell: &Shell, when: &When, rcfile: Option<String>) -> Result<()> {
    println!("# {when} for {shell}");
    match shell_init(shell, when, rcfile) {
        Ok(source) => println!("{source}"),
        Err(err) => println!("# Could not load source: {err}"),
    }
    Ok(())
}
