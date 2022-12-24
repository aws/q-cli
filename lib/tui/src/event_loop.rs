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
    Container,
    FilePickerEvent,
    Layout,
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
}

impl<'i, 'o> State<'i, 'o> {
    fn new(style_sheet: StyleSheet<'i, 'o>) -> Self {
        Self {
            style_sheet,
            event_buffer: vec![],
            tree: vec![],
            cursor_position: (0.0, 0.0),
            cursor_color: ColorAttribute::Default,
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
        component: C,
        input_method: &InputMethod,
        style_sheet: StyleSheet,
        mut event_handler: F,
    ) -> Result<(), Error>
    where
        C: Component + 'static,
        F: 'a + FnMut(Event, &mut dyn Component, &mut ControlFlow),
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

        let mut component = Container::new("", Layout::Vertical).push(component);

        let mut state = State::new(style_sheet);
        component.on_focus(&mut state, true);

        let mut control_flow = ControlFlow::Wait;
        while let ControlFlow::Wait = control_flow {
            // todo: seems like there's an issue in termwiz which doesn't
            // account for grapheme width in optimized surface diffs
            for _ in 0..2 {
                surface.add_changes(vec![
                    Change::ClearScreen(ColorAttribute::Default),
                    Change::CursorVisibility(CursorVisibility::Hidden),
                ]);
                component.draw(&mut state, &mut surface, 0.0, 0.0, cols, rows, cols, rows);

                buf.add_change(Change::CursorVisibility(CursorVisibility::Hidden));
                buf.draw_from_screen(&surface, 0, 0);
                buf.add_changes(vec![
                    Change::CursorPosition {
                        x: Position::Absolute(state.cursor_position.0.round() as usize),
                        y: Position::Absolute(state.cursor_position.1.round() as usize),
                    },
                    Change::CursorColor(state.cursor_color),
                    Change::CursorVisibility(surface.cursor_visibility()),
                ]);

                buf.flush()?;

                surface.flush_changes_older_than(surface.current_seqno());
            }

            self.handle_event(
                &mut component,
                input_method,
                &mut event_handler,
                buf.terminal().poll_input(None)?.unwrap(),
                &mut state,
                &mut control_flow,
                &mut surface,
                &mut buf,
                &mut cols,
                &mut rows,
            );

            while let Some(event) = buf.terminal().poll_input(Some(Duration::ZERO))? {
                self.handle_event(
                    &mut component,
                    input_method,
                    &mut event_handler,
                    event,
                    &mut state,
                    &mut control_flow,
                    &mut surface,
                    &mut buf,
                    &mut cols,
                    &mut rows,
                );
            }

            while let Some(event) = state.event_buffer.pop() {
                event_handler(event, &mut component, &mut control_flow);
            }
        }

        buf.terminal().set_cooked_mode()?;
        buf.terminal().flush()?;

        Ok(())
    }

    pub fn handle_event<'a, F>(
        &self,
        component: &mut Container,
        input_method: &InputMethod,
        event_handler: &mut F,
        event: InputEvent,
        state: &mut State,
        control_flow: &mut ControlFlow,
        surface: &mut Surface,
        buf: &mut BufferedTerminal<impl Terminal>,
        cols: &mut f64,
        rows: &mut f64,
    ) where
        F: 'a + FnMut(Event, &mut dyn Component, &mut ControlFlow),
    {
        match event {
            InputEvent::Key(event) => {
                let input_action = input_method.get_action(event);
                match input_action {
                    InputAction::Submit => {
                        component.on_input_action(state, &input_action);
                        if component.next(state, false).is_none() {
                            *control_flow = ControlFlow::Quit;
                        }
                    },
                    InputAction::Next => match component.next(state, true) {
                        Some(id) => event_handler(Event::FocusChanged { id, focus: true }, component, control_flow),
                        None => *control_flow = ControlFlow::Quit,
                    },
                    InputAction::Previous => {
                        if let Some(id) = component.prev(state, true) {
                            event_handler(Event::FocusChanged { id, focus: true }, component, control_flow)
                        }
                    },
                    InputAction::Quit => event_handler(Event::Quit, component, control_flow),
                    InputAction::Terminate => event_handler(Event::Terminate, component, control_flow),
                    InputAction::TempChangeView => {
                        component.on_focus(state, false);
                        event_handler(Event::TempChangeView, component, control_flow);
                        component.on_focus(state, true);
                    },
                    _ => component.on_input_action(state, &input_action),
                }
            },
            InputEvent::Mouse(mut event) => {
                event.x -= 1;
                event.y -= 1;
                component.on_mouse_event(state, &event, 0.0, 0.0, *cols, *rows);
            },
            InputEvent::Resized {
                cols: ncols,
                rows: nrows,
            } => {
                surface.resize(ncols, nrows);
                buf.add_change(Change::ClearScreen(ColorAttribute::Default));
                buf.resize(ncols, nrows);

                *cols = ncols as f64;
                *rows = nrows as f64;
            },
            InputEvent::Paste(clipboard) => component.on_input_action(state, &InputAction::Paste(clipboard)),
            _ => (),
        }
    }
}
