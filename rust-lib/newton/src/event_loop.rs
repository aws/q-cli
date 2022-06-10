use std::io::{
    stdout,
    Stdout,
    Write,
};
use std::time::Duration;

use crossterm::cursor::{self,};
use crossterm::event::{
    poll,
    read,
};
use crossterm::style::{
    Color,
    Colors,
    SetColors,
};
use crossterm::terminal::{
    self,
    disable_raw_mode,
    enable_raw_mode,
    Clear,
    ClearType,
    EnterAlternateScreen,
    LeaveAlternateScreen,
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
    Exit,
}

pub struct EventLoop {
    out: Stdout,
}
impl Default for EventLoop {
    fn default() -> Self {
        Self::new()
    }
}

impl EventLoop {
    pub fn new() -> Self {
        Self { out: stdout() }
    }

    pub fn run<F, E>(&mut self, mut control_flow: ControlFlow, display_mode: DisplayMode, mut func: F) -> Result<(), E>
    where
        F: FnMut(Event, &mut DisplayState, &mut ControlFlow) -> Result<(), E>,
        E: From<std::io::Error>,
    {
        if let DisplayMode::AlternateScreen = display_mode {
            self.out
                .queue(EnterAlternateScreen)?
                .queue(Clear(ClearType::All))?
                .queue(cursor::MoveTo(0, 0))?;
        };
        enable_raw_mode()?;
        self.out
            .queue(cursor::Hide)?
            .queue(SetColors(Colors::new(Color::Reset, Color::Reset)))?;
        self.out.flush()?;

        let size = terminal::size()?;
        let mut display_state = DisplayState::new(terminal::size()?);
        func(
            Event::Initialize {
                width: size.0,
                height: size.1,
            },
            &mut display_state,
            &mut control_flow,
        )?;
        func(Event::Update, &mut display_state, &mut control_flow)?;
        func(Event::Draw, &mut display_state, &mut control_flow)?;
        display_state.write_diff(&mut self.out)?;

        loop {
            match control_flow {
                ControlFlow::Poll => {
                    while let true = poll(Duration::ZERO)? {
                        let event = match Event::from(read()?) {
                            event @ Event::Resized { width, height } => {
                                display_state.resize(width, height);
                                event
                            },
                            event => event,
                        };

                        func(event, &mut display_state, &mut control_flow)?;
                    }
                },
                ControlFlow::Wait => {
                    let event = match Event::from(read()?) {
                        event @ Event::Resized { width, height } => {
                            display_state.resize(width, height);
                            event
                        },
                        event => event,
                    };

                    func(event, &mut display_state, &mut control_flow)?;

                    while let true = poll(Duration::ZERO)? {
                        let event = match Event::from(read()?) {
                            event @ Event::Resized { width, height } => {
                                display_state.resize(width, height);
                                event
                            },
                            event => event,
                        };

                        func(event, &mut display_state, &mut control_flow)?;
                    }
                },
                ControlFlow::Exit => break,
            }

            func(Event::Update, &mut display_state, &mut control_flow)?;
            func(Event::Draw, &mut display_state, &mut control_flow)?;
            display_state.write_diff(&mut self.out)?;
        }

        disable_raw_mode()?;

        #[allow(clippy::single_match)]
        match display_mode {
            DisplayMode::AlternateScreen => {
                self.out.queue(LeaveAlternateScreen)?;
            },
            _ => (),
        }

        self.out
            .queue(cursor::Show)?
            .queue(SetColors(Colors::new(Color::Reset, Color::Reset)))?
            .flush()?;

        Ok(())
    }
}
