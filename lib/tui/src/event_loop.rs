use std::time::Duration;

use termwiz::caps::Capabilities;
use termwiz::color::ColorAttribute;
use termwiz::input::InputEvent;
use termwiz::surface::{
    Change,
    CursorVisibility,
    Surface,
};
use termwiz::terminal::{
    new_terminal,
    Terminal,
};

use crate::component::{
    CheckBoxEvent,
    Component,
    FilePickerEvent,
    SelectEvent,
    TextFieldEvent,
};
use crate::input::InputAction;
use crate::{
    Error,
    InputMethod,
    StyleSheet,
};

#[derive(Debug)]
pub struct State {
    pub style_sheet: StyleSheet,
    pub event_buffer: Vec<Event>,
}

impl State {
    fn new(style_sheet: StyleSheet) -> Self {
        Self {
            style_sheet,
            event_buffer: vec![],
        }
    }
}

#[derive(Debug)]
pub enum Event {
    Quit,
    Terminate,
    MainEventsCleared,
    // todo(chay): remove
    TempChangeView,
    FocusChanged { id: String, focus: bool },
    HoverChanged { id: String, hover: bool },
    ActiveChanged { id: String, active: bool },
    CheckBox(CheckBoxEvent),
    FilePicker(FilePickerEvent),
    Select(SelectEvent),
    TextField(TextFieldEvent),
}

#[derive(Clone, Copy, Debug)]
pub enum ControlFlow {
    Wait,
    Quit,
}

#[derive(Debug, Default)]
pub struct EventLoop;

impl EventLoop {
    pub fn new() -> Self {
        Self::default()
    }

    #[inline]
    pub fn run<'a, C, F>(
        &self,
        component: &'a mut C,
        input_method: InputMethod,
        style_sheet: StyleSheet,
        mut event_handler: F,
    ) -> Result<(), Error>
    where
        C: Component,
        F: 'a + FnMut(Event, &mut C, &mut ControlFlow),
    {
        let mut terminal = new_terminal(Capabilities::new_from_env()?)?;
        terminal.enter_alternate_screen()?;

        terminal.set_raw_mode()?;
        terminal.render(&[Change::CursorVisibility(CursorVisibility::Hidden)])?;

        let screen_size = terminal.get_screen_size()?;
        let mut cols = screen_size.cols as f64;
        let mut rows = screen_size.rows as f64;

        let mut surface = Surface::new(screen_size.cols, screen_size.rows);
        let mut backbuffer = Surface::new(screen_size.cols, screen_size.rows);

        let mut state = State::new(style_sheet);
        component.initialize(&mut state);
        component.on_focus(&mut state, true);

        let mut scripted_events = match &input_method {
            InputMethod::Scripted(input_events) => input_events.iter().cloned().rev().collect(),
            _ => vec![],
        };

        let mut control_flow = ControlFlow::Wait;
        loop {
            // drawing code
            surface.add_change(Change::ClearScreen(ColorAttribute::Default));
            let style = component.style(&mut state);
            component.draw(
                &mut state,
                &mut surface,
                style.spacing_left(),
                style.spacing_top(),
                cols - style.spacing_horizontal(),
                rows - style.spacing_vertical(),
                cols,
                rows,
            );

            // diffing logic
            let diff = backbuffer.diff_screens(&surface);
            if !diff.is_empty() {
                terminal.render(&diff)?;

                let mut seq = backbuffer.add_changes(diff);
                backbuffer.flush_changes_older_than(seq);

                seq = surface.current_seqno();
                surface.flush_changes_older_than(seq);
            }

            let duration = match control_flow {
                ControlFlow::Wait => Some(Duration::from_millis(16)),
                ControlFlow::Quit => break,
            };

            while let Some(event) = terminal.poll_input(duration)?.or_else(|| scripted_events.pop()) {
                match event {
                    InputEvent::Key(event) => {
                        let code = event.key;
                        let modifiers = event.modifiers;

                        for input_action in InputAction::from_key(&input_method, code, modifiers) {
                            match input_action {
                                InputAction::Submit => {
                                    if component.on_input_action(&mut state, input_action)
                                        && component.next(&mut state, false).is_none()
                                    {
                                        control_flow = ControlFlow::Quit;
                                    }
                                },
                                InputAction::Next => match component.next(&mut state, true) {
                                    Some(id) => event_handler(
                                        Event::FocusChanged { id, focus: true },
                                        component,
                                        &mut control_flow,
                                    ),
                                    None => control_flow = ControlFlow::Quit,
                                },
                                InputAction::Previous => {
                                    if let Some(id) = component.prev(&mut state, true) {
                                        event_handler(
                                            Event::FocusChanged { id, focus: true },
                                            component,
                                            &mut control_flow,
                                        )
                                    }
                                },
                                InputAction::Quit => event_handler(Event::Quit, component, &mut control_flow),
                                InputAction::Terminate => event_handler(Event::Terminate, component, &mut control_flow),
                                InputAction::ChangeView => {
                                    component.on_focus(&mut state, false);
                                    event_handler(Event::TempChangeView, component, &mut control_flow);
                                    component.initialize(&mut state);
                                    component.on_focus(&mut state, true);
                                },
                                _ => {
                                    component.on_input_action(&mut state, input_action);
                                },
                            }
                        }
                    },
                    // todo(chay): add back
                    // InputEvent::Mouse(event) => component.on_mouse_event(&mut state, &event, 0.0, 0.0, cols, rows),
                    InputEvent::Resized {
                        cols: new_cols,
                        rows: new_rows,
                    } => {
                        cols = new_cols as f64;
                        rows = new_rows as f64;

                        surface.resize(new_cols, new_rows);
                        backbuffer.resize(new_cols, new_rows);

                        component.on_resize(&mut state, cols, rows);

                        surface.add_change(Change::ClearScreen(ColorAttribute::PaletteIndex(0)));
                        backbuffer.add_change(Change::ClearScreen(ColorAttribute::Default));
                        terminal.render(&[Change::ClearScreen(ColorAttribute::Default)])?;
                    },
                    _ => (),
                }
            }

            while let Some(event) = state.event_buffer.pop() {
                event_handler(event, component, &mut control_flow);
            }
        }

        terminal.flush()?;

        Ok(())
    }
}
