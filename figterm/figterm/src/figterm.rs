use std::{
    env::{current_dir, set_current_dir},
    path::Path,
};

use tokio::sync::mpsc::UnboundedSender;

use crate::{
    command_info::CommandInfo,
    fig_info::FigInfo,
    new_history::History,
    proto::{
        hooks::{hook_to_message, new_context, new_preexec_hook, new_prompt_hook},
        ShellContext,
    },
    screen::FigtermScreen,
};

struct ShellState {
    tty: Option<String>,
    pid: Option<i32>,
    session_id: Option<String>,
    hostname: Option<String>,

    shell: Option<String>,

    in_ssh: bool,
    in_docker: bool,

    preexec: bool,
    in_prompt: bool,
}

impl ShellState {
    fn new() -> ShellState {
        ShellState {
            tty: None,
            pid: None,
            session_id: None,
            hostname: None,
            shell: None,
            in_ssh: false,
            in_docker: false,
            preexec: false,
            in_prompt: false,
        }
    }
}

pub struct Figterm {
    fig_info: FigInfo,
    shell_state: ShellState,
    pub screen: FigtermScreen,

    history: Option<History>,

    last_command: Option<CommandInfo>,
    has_seen_prompt: bool,

    unix_socket_sender: UnboundedSender<Vec<u8>>,
}

impl Figterm {
    pub fn new(sender: UnboundedSender<Vec<u8>>, fig_info: FigInfo) -> Figterm {
        Figterm {
            unix_socket_sender: sender,

            fig_info,
            screen: FigtermScreen::new(),
            shell_state: ShellState::new(),

            history: History::load().ok(),

            has_seen_prompt: false,
            last_command: None,
        }
    }

    pub fn get_context(&self) -> ShellContext {
        new_context(
            self.shell_state.pid,
            self.shell_state.tty.clone(),
            self.shell_state.shell.clone(),
            None,
            self.fig_info.term_session_id.clone(),
            self.fig_info.fig_integration_version,
            None,
            None,
        )
    }

    pub fn resize(&mut self, width: usize, height: usize) {
        self.screen.resize(width, height);
    }
}

impl ansi::Handler for Figterm {
    fn set_title(&mut self, _: Option<String>) {}

    fn set_cursor_style(&mut self, _: Option<ansi::CursorStyle>) {}

    fn set_cursor_shape(&mut self, _shape: ansi::CursorShape) {}

    fn input(&mut self, c: char) {
        self.screen.write(c);
        log::info!("Cursor: {}", self.screen.cursor);
    }

    fn goto(&mut self, line: Line, column: Column) {
        self.screen.goto(line, column);
        log::info!("Cursor: {}", self.screen.cursor);
    }

    fn goto_line(&mut self, line: Line) {
        self.screen.goto_line(line);
        log::info!("Cursor: {}", self.screen.cursor);
    }

    fn goto_col(&mut self, column: Column) {
        self.screen.goto_column(column);
        log::info!("Cursor: {}", self.screen.cursor);
    }

    fn insert_blank(&mut self, n: usize) {
        for _ in 0..n {
            // self.screen.insert_blank();
        }
    }

    fn move_up(&mut self, n: usize) {
        self.screen.move_up(n);
        log::info!("Cursor: {}", self.screen.cursor);
    }

    fn move_down(&mut self, n: usize) {
        self.screen.move_down(n);
        log::info!("Cursor: {}", self.screen.cursor);
    }

    fn identify_terminal(&mut self, _intermediate: Option<char>) {}

    fn device_status(&mut self, _: usize) {}

    fn move_forward(&mut self, columns: Column) {
        self.screen.move_forward(columns);
        log::info!("Cursor: {}", self.screen.cursor);
    }

    fn move_backward(&mut self, columns: Column) {
        self.screen.move_backward(columns);
        log::info!("Cursor: {}", self.screen.cursor);
    }

    fn move_down_and_cr(&mut self, n: usize) {
        self.screen.move_down(n);
        self.screen.goto_column(1);
        log::info!("Cursor: {}", self.screen.cursor);
    }

    fn move_up_and_cr(&mut self, n: usize) {
        self.screen.move_up(n);
        self.screen.goto_column(1);
        log::info!("Cursor: {}", self.screen.cursor);
    }

    fn put_tab(&mut self, _count: u16) {}

    fn backspace(&mut self) {}

    fn carriage_return(&mut self) {
        self.screen.goto_column(1);
    }

    fn linefeed(&mut self) {
        self.screen.move_down(1);
    }

    fn bell(&mut self) {}

    fn substitute(&mut self) {}

    fn newline(&mut self) {
        self.screen.move_down(1);
        self.screen.goto_column(1);
    }

    fn set_horizontal_tabstop(&mut self) {}

    fn scroll_up(&mut self, _: usize) {}

    fn scroll_down(&mut self, _: usize) {}

    fn insert_blank_lines(&mut self, _: usize) {}

    fn delete_lines(&mut self, _: usize) {}

    fn erase_chars(&mut self, _: Column) {}

    fn delete_chars(&mut self, _: usize) {}

    fn move_backward_tabs(&mut self, _count: u16) {}

    fn move_forward_tabs(&mut self, _count: u16) {}

    fn save_cursor_position(&mut self) {}

    fn restore_cursor_position(&mut self) {}

    fn clear_line(&mut self, _mode: ansi::LineClearMode) {}

    fn clear_screen(&mut self, _mode: ansi::ClearMode) {}

    fn clear_tabs(&mut self, _mode: ansi::TabulationClearMode) {}

    fn reset_state(&mut self) {}

    fn reverse_index(&mut self) {}

    fn terminal_attribute(&mut self, _attr: ansi::Attr) {}

    fn set_mode(&mut self, _mode: ansi::Mode) {}

    fn unset_mode(&mut self, _: ansi::Mode) {}

    fn set_scrolling_region(&mut self, _top: usize, _bottom: Option<usize>) {}

    fn set_keypad_application_mode(&mut self) {}

    fn unset_keypad_application_mode(&mut self) {}

    fn set_active_charset(&mut self, _: ansi::CharsetIndex) {}

    fn configure_charset(&mut self, _: ansi::CharsetIndex, _: ansi::StandardCharset) {}

    fn set_color(&mut self, _: usize, _: ansi::Rgb) {}

    fn dynamic_color_sequence(&mut self, _: u8, _: usize, _: &str) {}

    fn reset_color(&mut self, _: usize) {}

    fn clipboard_store(&mut self, _: u8, _: &[u8]) {}

    fn clipboard_load(&mut self, _: u8, _: &str) {}

    fn decaln(&mut self) {}

    fn push_title(&mut self) {}

    fn pop_title(&mut self) {}

    fn text_area_size_pixels(&mut self) {}

    fn text_area_size_chars(&mut self) {}

    fn new_cmd(&mut self) {
        let context = self.get_context();
        let hook = new_prompt_hook(Some(context));
        let message = hook_to_message(hook).to_fig_pbuf();
        match self.unix_socket_sender.send(message) {
            Ok(_) => (),
            Err(e) => log::error!("Failed to queue `Prompt` hook: {}", e),
        }

        log::info!("Prompt at position: {}", self.screen.cursor);

        if let Some(command) = &self.last_command {
            if let Some(history) = &mut self.history {
                match history.insert_command_history(command.clone()) {
                    Ok(_) => {}
                    Err(e) => log::error!("Failed to insert command history: {}", e),
                }
            }
            log::info!("{:?}", command);
        }
    }

    fn start_prompt(&mut self) {
        self.has_seen_prompt = true;

        self.shell_state.in_prompt = true;
        self.screen.screen_attribs.in_prompt = true;
    }

    fn end_prompt(&mut self) {
        self.shell_state.in_prompt = false;
        self.screen.screen_attribs.in_prompt = false;
    }

    fn pre_exec(&mut self) {
        // Send PreExec hook
        let context = self.get_context();
        let hook = new_preexec_hook(Some(context));
        let message = hook_to_message(hook).to_fig_pbuf();
        match self.unix_socket_sender.send(message) {
            Ok(_) => {}
            Err(e) => log::error!("Failed to queue `PreExec` hook: {}", e),
        }

        self.last_command = Some(CommandInfo {
            command: String::new(),
            shell: self.shell_state.shell.clone(),
            pid: self.shell_state.pid,
            session_id: self.fig_info.term_session_id.clone(),
            cwd: current_dir().ok(),
            time: std::time::SystemTime::now()
                .duration_since(std::time::SystemTime::UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs(),
            in_ssh: self.shell_state.in_ssh,
            in_docker: self.shell_state.in_docker,
            hostname: self.shell_state.shell.clone(),
            exit_code: None,
        });

        self.shell_state.preexec = true;
    }

    fn dir(&mut self, directory: &Path) {
        log::info!("In dir {}", directory.display());
        match set_current_dir(directory) {
            Ok(_) => {}
            Err(e) => log::error!("Failed to set current dir: {}", e),
        }
    }

    fn exit_code(&mut self, exit_code: i32) {
        if let Some(command) = self.last_command.as_mut() {
            command.exit_code = Some(exit_code);
        }
    }

    fn shell(&mut self, shell: &str) {
        self.shell_state.shell = Some(shell.to_owned());
    }

    fn fish_suggestion_color(&mut self, _: &str) {}

    fn zsh_suggestion_color(&mut self, _: &str) {}

    fn tty(&mut self, tty: &str) {
        self.shell_state.tty = Some(tty.to_owned());
    }

    fn pid(&mut self, pid: i32) {
        self.shell_state.pid = Some(pid);
    }

    fn session_id(&mut self, session_id: &str) {
        self.shell_state.session_id = Some(session_id.to_owned());
    }

    fn docker(&mut self, docker: bool) {
        self.shell_state.in_docker = docker;
    }

    fn ssh(&mut self, ssh: bool) {
        self.shell_state.in_ssh = ssh;
    }

    fn hostname(&mut self, hostname: &str) {
        self.shell_state.hostname = Some(hostname.to_owned());
    }

    fn log(&mut self, _: &str) {}
}
