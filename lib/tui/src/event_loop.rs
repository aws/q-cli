use std::time::Duration;

use lightningcss::stylesheet::StyleSheet;
use termwiz::caps::Capabilities;
use termwiz::color::ColorAttribute;
use termwiz::input::InputEvent;
use termwiz::surface::{
    Change,
    CursorShape,
    CursorVisibility,
    Position,
    Surface,
};
use termwiz::terminal::buffered::BufferedTerminal;
use termwiz::terminal::{
    new_terminal,
    Terminal,
};

use crate::component::{
    CheckBoxEvent,
    Component,
    FilePickerEvent,
    SelectEvent,
    StyleInfo,
    TextFieldEvent,
};
use crate::input::InputAction;
use crate::{
    Error,
    InputMethod,
};

#[derive(Debug, Clone)]
pub struct TreeElement {
    pub inner: StyleInfo,
    pub siblings: std::collections::LinkedList<StyleInfo>,
}

impl TreeElement {
    pub fn next_sibling(self) -> Option<Self> {
        let mut siblings = self.siblings;
        let inner = siblings.pop_front()?;
        Some(Self { inner, siblings })
    }
}

pub struct State<'i, 'o> {
    pub style_sheet: StyleSheet<'i, 'o>,
    pub event_buffer: Vec<Event>,
    pub tree: Vec<TreeElement>,
    pub cursor_position: (f64, f64),
    pub cursor_color: ColorAttribute,
    pub cursor_visibility: bool,
}

impl<'i, 'o> State<'i, 'o> {
    fn new(style_sheet: StyleSheet<'i, 'o>) -> Self {
        Self {
            style_sheet,
            event_buffer: vec![],
            tree: vec![],
            cursor_position: (0.0, 0.0),
            cursor_color: ColorAttribute::Default,
            cursor_visibility: false,
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
        let capabilities = Capabilities::new_from_env()?;
        let mut buf = BufferedTerminal::new(new_terminal(capabilities)?)?;
        buf.terminal().enter_alternate_screen()?;
        buf.terminal().set_raw_mode()?;
        buf.add_change(Change::CursorShape(CursorShape::BlinkingBar));

        let screen_size = buf.terminal().get_screen_size()?;
        let mut cols = screen_size.cols as f64;
        let mut rows = screen_size.rows as f64;

        let mut surface = Surface::new(screen_size.cols, screen_size.rows);

        let mut state = State::new(style_sheet);
        state.tree.push(TreeElement {
            inner: component.inner().style_info(),
            siblings: Default::default(),
        });

        component.initialize(&mut state);
        component.on_focus(&mut state, true);

        let mut scripted_events = match &input_method {
            InputMethod::Scripted(input_events) => input_events.iter().cloned().rev().collect(),
            _ => vec![],
        };

        let mut control_flow = ControlFlow::Wait;
        while let ControlFlow::Wait = control_flow {
            // todo: seems like there's an issue in termwiz which doesn't
            // account for grapheme width in optimized surface diffs
            for _ in 0..2 {
                let style = component.style(&state);
                // todo: this doesn't work, why?
                // if let Display::None = style.display() {
                //    continue;
                //}

                surface.add_change(Change::ClearScreen(ColorAttribute::Default));
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

                buf.add_change(Change::CursorVisibility(CursorVisibility::Hidden));
                buf.draw_from_screen(&surface, 0, 0);
                buf.add_changes(vec![
                    Change::CursorPosition {
                        x: Position::Absolute(state.cursor_position.0.round() as usize),
                        y: Position::Absolute(state.cursor_position.1.round() as usize),
                    },
                    Change::CursorColor(state.cursor_color),
                    Change::CursorVisibility(match state.cursor_visibility {
                        true => CursorVisibility::Visible,
                        false => CursorVisibility::Hidden,
                    }),
                ]);

                buf.flush()?;

                surface.flush_changes_older_than(surface.current_seqno());
            }

            let event = scripted_events.pop().or(buf.terminal().poll_input(None)?).unwrap();
            match event {
                InputEvent::Key(event) => {
                    let code = event.key;
                    let modifiers = event.modifiers;

                    for input_action in InputAction::from_key(&input_method, code, modifiers) {
                        tracing::error!("Got action {:?}", input_action);
                        match input_action {
                            InputAction::Submit => {
                                if component.on_input_action(&mut state, input_action).unwrap_or(false)
                                    && component.next(&mut state, false).is_none()
                                {
                                    control_flow = ControlFlow::Quit;
                                }
                            },
                            InputAction::Next => match component.next(&mut state, true) {
                                Some(id) => {
                                    event_handler(Event::FocusChanged { id, focus: true }, component, &mut control_flow)
                                },
                                None => control_flow = ControlFlow::Quit,
                            },
                            InputAction::Previous => {
                                if let Some(id) = component.prev(&mut state, true) {
                                    event_handler(Event::FocusChanged { id, focus: true }, component, &mut control_flow)
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
                    cols: ncols,
                    rows: nrows,
                } => {
                    surface.resize(ncols, nrows);
                    buf.add_change(Change::ClearScreen(ColorAttribute::Default));
                    buf.resize(ncols, nrows);

                    cols = ncols as f64;
                    rows = nrows as f64;
                },
                InputEvent::Paste(clipboard) => component.on_paste(&mut state, &clipboard),
                _ => (),
            }

            // todo(chay) this is literally copy pasted from above because writing a function for this takes
            // forever
            while let Some(event) = scripted_events
                .pop()
                .or(buf.terminal().poll_input(Some(Duration::ZERO))?)
            {
                match event {
                    InputEvent::Key(event) => {
                        let code = event.key;
                        let modifiers = event.modifiers;

                        for input_action in InputAction::from_key(&input_method, code, modifiers) {
                            tracing::error!("Got action {:?}", input_action);
                            match input_action {
                                InputAction::Submit => {
                                    if component.on_input_action(&mut state, input_action).unwrap_or(false)
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
                        cols: ncols,
                        rows: nrows,
                    } => {
                        surface.resize(ncols, nrows);
                        buf.add_change(Change::ClearScreen(ColorAttribute::Default));
                        buf.resize(ncols, nrows);

                        cols = ncols as f64;
                        rows = nrows as f64;
                    },
                    InputEvent::Paste(clipboard) => component.on_paste(&mut state, &clipboard),
                    _ => (),
                }
            }

            while let Some(event) = state.event_buffer.pop() {
                event_handler(event, component, &mut control_flow);
            }
        }

        buf.terminal().set_cooked_mode()?;
        buf.terminal().flush()?;

        Ok(())
    }
}
