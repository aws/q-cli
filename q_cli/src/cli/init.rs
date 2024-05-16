use std::borrow::Cow;
use std::fmt::Display;
use std::io::{
    stdout,
    Write,
};
use std::path::Path;
use std::process::ExitCode;

use auth::AMZN_START_URL;
use clap::Args;
use crossterm::style::Stylize;
use eyre::Result;
use fig_integrations::shell::{
    ShellExt,
    When,
};
use fig_util::env_var::Q_SHELL;
use fig_util::{
    get_parent_process_exe,
    Shell,
    Terminal,
    CLI_BINARY_NAME,
    PRODUCT_NAME,
};
use indoc::formatdoc;
use once_cell::sync::Lazy;

use super::internal::should_figterm_launch::should_figterm_launch_exit_status;
use crate::util::app_path_from_bundle_id;

static IS_SNAPSHOT_TEST: Lazy<bool> = Lazy::new(|| std::env::var_os("Q_INIT_SNAPSHOT_TEST").is_some());

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
}

impl InitArgs {
    pub async fn execute(&self) -> Result<ExitCode> {
        let InitArgs { shell, when, rcfile } = self;
        match shell_init(shell, when, rcfile).await {
            Ok(source) => writeln!(stdout(), "{source}"),
            Err(err) => writeln!(stdout(), "# Could not load source: {err}"),
        }
        .ok();
        Ok(ExitCode::SUCCESS)
    }
}

#[derive(PartialEq, Eq)]
enum GuardAssignment {
    BeforeSourcing,
    AfterSourcing,
}

fn assign_shell_variable(shell: &Shell, name: impl Display, value: impl Display, exported: bool) -> String {
    match (shell, exported) {
        (Shell::Bash | Shell::Zsh, false) => format!("{name}=\"{value}\""),
        (Shell::Bash | Shell::Zsh, true) => format!("export {name}=\"{value}\""),
        (Shell::Fish, false) => format!("set -g {name} \"{value}\""),
        (Shell::Fish, true) => format!("set -gx {name} \"{value}\""),
        (Shell::Nu, _) => format!("let-env {name} = \"{value}\";"),
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
        Shell::Nu => format!("if env | any name == '{guard_var}' {{").into(),
    });

    let shell_var = assign_shell_variable(shell, guard_var, "1", export);
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
            Shell::Nu => "}",
        }
        .into(),
    );

    output.join("\n")
}

async fn shell_init(shell: &Shell, when: &When, rcfile: &Option<String>) -> Result<String> {
    // Do not print any shell integrations for `.profile` as it can cause issues on launch
    if std::env::consts::OS == "linux" && matches!(rcfile.as_deref(), Some("profile")) {
        return Ok("".into());
    }

    if !fig_settings::state::get_bool_or("shell-integrations.enabled", true) {
        return Ok(guard_source(
            shell,
            false,
            "Q_SHELL_INTEGRATION_DISABLED",
            GuardAssignment::AfterSourcing,
            "echo '[Debug]: fig shell integration is disabled.'",
        ));
    }

    let mut to_source = Vec::new();

    if let Some(parent_process) = get_parent_process_exe() {
        to_source.push(assign_shell_variable(
            shell,
            Q_SHELL,
            if *IS_SNAPSHOT_TEST {
                Path::new("/bin/zsh").display()
            } else {
                parent_process.display()
            },
            false,
        ));
    };

    if when == &When::Pre {
        let status = if *IS_SNAPSHOT_TEST {
            0
        } else {
            should_figterm_launch_exit_status(true)
        };
        to_source.push(assign_shell_variable(shell, "SHOULD_QTERM_LAUNCH", status, false));
    }

    // Grabbing the real auth is too slow here, so we just rely on the cached value
    let is_amzn_user = fig_settings::state::get_string_or("auth.idc.start-url", "") == AMZN_START_URL;
    let inline_enabled = fig_settings::settings::get_bool_or("inline.enabled", is_amzn_user);

    if let When::Post = when {
        if !matches!(
            (shell, rcfile.as_deref()),
            (Shell::Zsh, Some("zprofile")) | (Shell::Bash, Some("profile" | "bash_profile"))
        ) && fig_settings::state::get_bool_or("dotfiles.enabled", true)
            && shell == &Shell::Zsh
            && when == &When::Post
            && inline_enabled
            && !*IS_SNAPSHOT_TEST
        {
            to_source.push(guard_source(
                shell,
                false,
                "Q_DOTFILES_SOURCED",
                GuardAssignment::AfterSourcing,
                fig_integrations::shell::inline_shell_completion_plugin::ZSH_SCRIPT,
            ));
        }

        // if stdin().is_tty() && env::var_os(PROCESS_LAUNCHED_BY_Q).is_none() {
        //     // if no value, assume that we have seen onboarding already.
        //     // this is explicitly set in onboarding in macOS app.
        //     let has_seen_onboarding: bool = fig_settings::state::get_bool_or("user.onboarding", true);

        //     if is_logged_in().await && !has_seen_onboarding {
        //         to_source.push("fig app onboarding".into())
        //     }
        // }

        if fig_settings::state::get_bool_or("shell-integrations.immediateLogin", false)
            && fig_settings::state::set_value("shell-integrations.immediateLogin", false).is_ok()
        {
            to_source.push(format!("{CLI_BINARY_NAME} login"));
        }
    }

    let is_jetbrains_terminal = Terminal::is_jetbrains_terminal();

    if when == &When::Pre && shell == &Shell::Bash && is_jetbrains_terminal {
        // JediTerm does not launch as a 'true' login shell, so our normal "shopt -q login_shell" check does
        // not work. Thus, Q_IS_LOGIN_SHELL will be incorrect. We must manually set it so the
        // user's bash_profile is sourced. https://github.com/JetBrains/intellij-community/blob/master/plugins/terminal/resources/jediterm-bash.in
        to_source.push("Q_IS_LOGIN_SHELL=1".into());
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

                // We source both the old and new location of these integrations.
                // In theory, they shouldn't both exist since they come with the app bundle itself.
                // As of writing, the bash path change isn't live, but we source it anyway.
                match shell {
                    Shell::Bash => Some(formatdoc! {"
                        [ -f '{bundle}/Contents/plugins/terminal/jediterm-bash.in' ] && source '{bundle}/Contents/plugins/terminal/jediterm-bash.in'
                        [ -f '{bundle}/Contents/plugins/terminal/bash/jediterm-bash.in' ] && source '{bundle}/Contents/plugins/terminal/bash/jediterm-bash.in'
                    "}),
                    Shell::Zsh => Some(formatdoc! {"
                        [ -f '{bundle}/Contents/plugins/terminal/.zshenv' ] && source '{bundle}/Contents/plugins/terminal/.zshenv'
                        [ -f '{bundle}/Contents/plugins/terminal/zsh/.zshenv' ] && source '{bundle}/Contents/plugins/terminal/zsh/.zshenv'
                    "}),
                    Shell::Fish => Some(formatdoc! {"
                        [ -f '{bundle}/Contents/plugins/terminal/fish/config.fish' ] && source '{bundle}/Contents/plugins/terminal/fish/config.fish'
                        [ -f '{bundle}/Contents/plugins/terminal/fish/init.fish' ] && source '{bundle}/Contents/plugins/terminal/fish/init.fish'
                    "}),
                    Shell::Nu => None,
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
                "Q_JETBRAINS_SHELL_INTEGRATION",
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
        if let Some(terminal) = Terminal::parent_terminal() {
            let prompt_state_key = format!("prompt.input-method.{}.count", terminal.internal_id());
            let prompt_count = fig_settings::state::get_int_or(&prompt_state_key, 0);
            if terminal.supports_macos_input_method() && prompt_count < 2 {
                let _ = fig_settings::state::set_value(&prompt_state_key, prompt_count + 1);
                to_source.push(input_method_prompt_code(*shell, &terminal));
            }
        }
    }

    if inline_enabled && when == &When::Post && shell == &Shell::Zsh {
        let key = "prompt.inline.count";
        if let Ok(prompt_count) = fig_settings::state::get_int(key) {
            let prompt_count = prompt_count.unwrap_or_default();
            if prompt_count < 1 {
                let _ = fig_settings::state::set_value(key, prompt_count + 1);
                to_source.push(inline_prompt_code(*shell));
            }
        }
    }

    Ok(to_source.join("\n"))
}

#[allow(dead_code)]
fn input_method_prompt_code(shell: Shell, terminal: &Terminal) -> String {
    guard_source(
        &shell,
        false,
        "Q_INPUT_METHOD_PROMPT",
        GuardAssignment::AfterSourcing,
        format!(
            "printf '\\n🚀 {PRODUCT_NAME} supports {terminal}!\\n\\nEnable integrations with {terminal} by \
             running:\\n  {}\\n\\n'\n",
            format!("{CLI_BINARY_NAME} integrations install input-method").magenta()
        ),
    )
}

fn inline_prompt_code(shell: Shell) -> String {
    guard_source(
        &shell,
        false,
        "Q_INLINE_PROMPT",
        GuardAssignment::AfterSourcing,
        format!(
            "printf '\\n{PRODUCT_NAME} now supports Inline AI suggestions!\\n\\nTo disable run: {}\\n\\n'\n",
            format!("{CLI_BINARY_NAME} settings inline.enabled false").magenta()
        ),
    )
}

#[cfg(test)]
mod tests {
    use std::process::{
        Command,
        Stdio,
    };

    use super::*;

    fn run_shell_stdout(shell: &Shell, text: &str) -> String {
        let mut child = Command::new(shell.as_str())
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .spawn()
            .unwrap();

        let stdin = child.stdin.as_mut().unwrap();
        stdin.write_all(text.as_bytes()).unwrap();
        stdin.flush().unwrap();

        let output = child.wait_with_output().unwrap();
        String::from_utf8(output.stdout).unwrap()
    }

    #[test]
    fn test_prompts() {
        for shell in [Shell::Bash, Shell::Zsh] {
            let terminal = Terminal::Iterm;
            let input_method_prompt_code = run_shell_stdout(&shell, &input_method_prompt_code(shell, &terminal));

            println!("=== input_method_prompt {shell:?} ===");
            println!("{input_method_prompt_code}");
            println!("===");

            assert_eq!(
                input_method_prompt_code,
                format!(
                    "\n🚀 {PRODUCT_NAME} supports {terminal}!\n\nEnable integrations with {terminal} by running:\n  {}\n\n",
                    format!("{CLI_BINARY_NAME} integrations install input-method").magenta()
                )
            );

            let inline_prompt_code = run_shell_stdout(&shell, &inline_prompt_code(shell));

            println!("=== inline_prompt {shell:?} ===");
            println!("{inline_prompt_code}");
            println!("===");

            assert_eq!(
                inline_prompt_code,
                format!(
                    "\n{PRODUCT_NAME} now supports Inline AI suggestions!\n\nTo disable run: {}\n\n",
                    format!("{CLI_BINARY_NAME} settings inline.enabled false").magenta()
                )
            );
        }
    }
}
