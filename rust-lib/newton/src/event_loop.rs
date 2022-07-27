use std::io::{
    stdout,
    Stdout,
    Write,
};
use std::time::Duration;

use crossterm::cursor::{self,};
use crossterm::event::{
    self,
    poll,
    read,
};
use crossterm::style::{
    self,
    Color,
    Colors,
    SetColors,
};
use crossterm::terminal::{
    self,
    disable_raw_mode,
    enable_raw_mode,
    ClearType,
};
use crossterm::QueueableCommand;

use crate::{
    DisplayState,
    Event,
};

#[derive(Clone, Copy, Debug)]
pub enum DisplayMode {
    Inline,
    AlternateScreen,
}

#[derive(Clone, Copy, Debug)]
pub enum ControlFlow {
    Poll,
    Wait,
    Exit(u32),
    Reenter(u32),
}

pub struct EventLoop {
    out: Stdout,
    initialized: bool,
    display_state: DisplayState,
    display_mode: DisplayMode,
}

impl EventLoop {
    pub fn new(display_mode: DisplayMode) -> Result<Self, std::io::Error> {
        let mut size = terminal::size()?;
        let cursor_position = match display_mode {
            DisplayMode::Inline => {
                let cursor_position = cursor::position()?;
                size.1 -= cursor_position.1;
                cursor_position
            },
            DisplayMode::AlternateScreen => (0, 0),
        };

        Ok(Self {
            out: stdout(),
            initialized: false,
            display_state: DisplayState::new(size, cursor_position.1),
            display_mode,
        })
    }

    pub fn run<F, E>(&mut self, mut control_flow: ControlFlow, mut func: F) -> Result<ControlFlow, E>
    where
        F: FnMut(Event, &mut DisplayState, &mut ControlFlow) -> Result<(), E>,
        E: From<std::io::Error>,
    {
        if !self.initialized {
            if let DisplayMode::AlternateScreen = self.display_mode {
                self.out
                    .queue(terminal::EnterAlternateScreen)?
                    .queue(terminal::Clear(ClearType::All))?
                    .queue(cursor::MoveTo(0, 0))?;
            }
            enable_raw_mode()?;
            self.out
                .queue(event::EnableMouseCapture)?
                .queue(cursor::Hide)?
                .queue(style::SetColors(Colors::new(Color::Reset, Color::Reset)))?;
            self.out.flush()?;

            func(
                Event::Initialize {
                    width: self.display_state.width(),
                    height: self.display_state.height(),
                },
                &mut self.display_state,
                &mut control_flow,
            )?;

            self.initialized = true;
        }

        func(Event::Update, &mut self.display_state, &mut control_flow)?;
        func(Event::Draw, &mut self.display_state, &mut control_flow)?;
        self.display_state.write_diff(&mut self.out)?;

        loop {
            match control_flow {
                ControlFlow::Poll => {
                    while let true = poll(Duration::ZERO)? {
                        let event = match Event::from(read()?) {
                            event @ Event::Resized { width, height } => {
                                self.display_state.resize(width, height)?;
                                event
                            },
                            event => event,
                        };

                        func(event, &mut self.display_state, &mut control_flow)?;
                    }
                },
                ControlFlow::Wait => {
                    let event = match Event::from(read()?) {
                        event @ Event::Resized { width, height } => {
                            self.display_state.resize(width, height)?;
                            event
                        },
                        event => event,
                    };

                    func(event, &mut self.display_state, &mut control_flow)?;

                    while let true = poll(Duration::ZERO)? {
                        let event = match Event::from(read()?) {
                            event @ Event::Resized { width, height } => {
                                self.display_state.resize(width, height)?;
                                event
                            },
                            event => event,
                        };

                        func(event, &mut self.display_state, &mut control_flow)?;
                    }
                },
                ControlFlow::Exit(_) => break,
                ControlFlow::Reenter(_) => return Ok(control_flow),
            }

            func(Event::Update, &mut self.display_state, &mut control_flow)?;
            func(Event::Draw, &mut self.display_state, &mut control_flow)?;
            self.display_state.write_diff(&mut self.out)?;
        }

        disable_raw_mode()?;

        #[allow(clippy::single_match)]
        match self.display_mode {
            DisplayMode::AlternateScreen => {
                self.out.queue(terminal::LeaveAlternateScreen)?;
            },
            _ => (),
        }

        self.out
            .queue(event::DisableMouseCapture)?
            .queue(cursor::Show)?
            .queue(SetColors(Colors::new(Color::Reset, Color::Reset)))?
            .flush()?;

        Ok(control_flow)
    }

    pub fn width(&self) -> i32 {
        self.display_state.width()
    }

    pub fn height(&self) -> i32 {
        self.display_state.height()
    }
}
