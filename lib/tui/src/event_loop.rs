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
};
use termwiz::terminal::{
    new_terminal,
    Terminal,
};

use crate::buffered_terminal::BufferedTerminal;
use crate::component::{
    CheckBoxEvent,
    Component,
    Div,
    FilePickerEvent,
    MultiselectEvent,
    SegmentedControlEvent,
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

#[cfg(debug_assertions)]
static mut CSS_STRING: String = String::new();

#[derive(Debug)]
pub enum DisplayMode {
    AlternateScreen,
    Inline,
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
    FocusChanged { id: String, focus: bool },
    HoverChanged { id: String, hover: bool },
    ActiveChanged { id: String, active: bool },
    CheckBox(CheckBoxEvent),
    FilePicker(FilePickerEvent),
    Multiselect(MultiselectEvent),
    SegmentedControl(SegmentedControlEvent),
    Select(SelectEvent),
    TextField(TextFieldEvent),
}

#[derive(Clone, Copy, Debug)]
pub enum ControlFlow {
    Wait,
    Poll(Duration),
    Quit,
}

pub struct EventLoop<'a> {
    component: Div,
    display_mode: DisplayMode,
    input_method: InputMethod,
    state: State<'a, 'a>,
    control_flow: ControlFlow,
    #[cfg(debug_assertions)]
    style_sheet_path: Option<std::path::PathBuf>,
}

impl<'a> EventLoop<'a> {
    pub fn new<C>(
        component: C,
        display_mode: DisplayMode,
        input_method: InputMethod,
        style_sheet: StyleSheet<'a, 'a>,
        control_flow: ControlFlow,
    ) -> Self
    where
        C: Component + 'static,
    {
        let mut component = Div::new().push(component);
        let mut state = State::new(style_sheet);
        component.on_focus(&mut state, true);

        Self {
            component,
            display_mode,
            input_method,
            state,
            control_flow,
            #[cfg(debug_assertions)]
            style_sheet_path: None,
        }
    }

    #[cfg(debug_assertions)]
    pub fn with_style_sheet_path(mut self, path: impl Into<std::path::PathBuf>) -> Self {
        self.style_sheet_path = Some(path.into());
        self
    }

    #[inline]
    pub fn run<'b, F>(&mut self, mut event_handler: F) -> Result<(), Error>
    where
        F: 'b + FnMut(Event, &mut dyn Component, &mut ControlFlow),
    {
        let capabilities = Capabilities::new_from_env()?;
        let mut buf = BufferedTerminal::new(new_terminal(capabilities)?)?;

        buf.terminal().set_raw_mode()?;
        buf.add_change(Change::CursorShape(CursorShape::BlinkingBar));

        let mut origin = match self.display_mode {
            DisplayMode::AlternateScreen => {
                buf.terminal().enter_alternate_screen()?;
                (0, 0)
            },
            DisplayMode::Inline => {
                // TODO: create custom error type
                let (x, y) = crossterm::cursor::position().unwrap();
                (x.into(), y.into())
            },
        };

        let screen_size = buf.terminal().get_screen_size()?;
        let mut screen_width = screen_size.cols;
        let mut screen_height = screen_size.rows;

        loop {
            // todo: seems like there's an issue in termwiz which doesn't
            // account for grapheme width in optimized surface diffs
            for _ in 0..1 {
                if let DisplayMode::Inline = self.display_mode {
                    let component_height = self.component.size(&mut self.state).1.round() as usize;
                    let remaining_height = screen_height.saturating_sub(origin.1);
                    let scroll_count = component_height.saturating_sub(remaining_height);
                    origin.1 = origin.1.saturating_sub(scroll_count);

                    if component_height < screen_height && scroll_count > 0 {
                        buf.add_change(Change::ClearScreen(ColorAttribute::Default));
                        buf.flush()?;
                        buf.terminal().render(&[Change::ScrollRegionUp {
                            first_row: 0,
                            region_size: screen_height,
                            scroll_count,
                        }])?;
                    }
                }

                buf.add_changes(vec![
                    Change::CursorVisibility(CursorVisibility::Hidden),
                    Change::ClearScreen(ColorAttribute::Default),
                ]);
                self.component.draw(
                    &mut self.state,
                    &mut buf,
                    0.0,
                    origin.1 as f64,
                    screen_width as f64,
                    screen_height as f64 - origin.1 as f64,
                );

                let cursor_visibility = buf.cursor_visibility();
                buf.add_changes(vec![
                    Change::CursorPosition {
                        x: Position::Absolute(self.state.cursor_position.0.round() as usize),
                        y: Position::Absolute(self.state.cursor_position.1.round() as usize),
                    },
                    Change::CursorColor(self.state.cursor_color),
                    Change::CursorVisibility(cursor_visibility),
                ]);
                buf.flush()?;
            }

            let event = match self.control_flow {
                ControlFlow::Wait => {
                    // Event can actually be `None` here despite blocking
                    let mut event = None;
                    while event.is_none() {
                        event = buf.terminal().poll_input(None)?;
                    }

                    event
                },
                ControlFlow::Poll(duration) => buf.terminal().poll_input(Some(duration))?,
                ControlFlow::Quit => break,
            };

            if let Some(event) = event {
                self.handle_event(
                    &mut event_handler,
                    event,
                    &mut buf,
                    &mut screen_width,
                    &mut screen_height,
                    origin.1 as f64,
                )?;
            }

            while let Some(event) = buf.terminal().poll_input(Some(Duration::ZERO))? {
                self.handle_event(
                    &mut event_handler,
                    event,
                    &mut buf,
                    &mut screen_width,
                    &mut screen_height,
                    origin.1 as f64,
                )?;
            }

            while let Some(event) = self.state.event_buffer.pop() {
                event_handler(event, &mut self.component, &mut self.control_flow);
            }

            event_handler(Event::MainEventsCleared, &mut self.component, &mut self.control_flow);
        }

        if let DisplayMode::Inline = self.display_mode {
            let component_height = self.component.size(&mut self.state).1;
            buf.add_change(Change::CursorPosition {
                x: Position::Absolute(0),
                y: Position::Absolute(origin.1 + component_height.round() as usize),
            });
            buf.flush()?;
        }

        Ok(())
    }

    pub fn handle_event<'b, F>(
        &mut self,
        event_handler: &mut F,
        event: InputEvent,
        buf: &mut BufferedTerminal<impl Terminal>,
        screen_width: &mut usize,
        screen_height: &mut usize,
        row_origin: f64,
    ) -> Result<(), Error>
    where
        F: 'b + FnMut(Event, &mut dyn Component, &mut ControlFlow),
    {
        match event {
            InputEvent::Key(event) => {
                let input_action = self.input_method.get_action(event);
                match input_action {
                    InputAction::Submit => {
                        self.component.on_input_action(&mut self.state, &input_action);
                        if self.component.next(&mut self.state, false).is_none() {
                            self.control_flow = ControlFlow::Quit;
                        }
                    },
                    InputAction::Next => match self.component.next(&mut self.state, true) {
                        Some(id) => event_handler(
                            Event::FocusChanged { id, focus: true },
                            &mut self.component,
                            &mut self.control_flow,
                        ),
                        None => self.control_flow = ControlFlow::Quit,
                    },
                    InputAction::Previous => {
                        if let Some(id) = self.component.prev(&mut self.state, true) {
                            event_handler(
                                Event::FocusChanged { id, focus: true },
                                &mut self.component,
                                &mut self.control_flow,
                            )
                        }
                    },
                    InputAction::Redraw => {
                        #[cfg(debug_assertions)]
                        if let Some(path) = &self.style_sheet_path {
                            // SAFETY: library must stay single threaded when in debug mode. This is a workaround to
                            // lightningcss limitations
                            unsafe {
                                CSS_STRING = std::fs::read_to_string(path)?;
                                self.state.style_sheet =
                                    StyleSheet::parse(&CSS_STRING, lightningcss::stylesheet::ParserOptions::default())
                                        .unwrap();
                            }
                        }
                    },
                    InputAction::Quit => {
                        self.component.on_focus(&mut self.state, false);
                        event_handler(Event::Quit, &mut self.component, &mut self.control_flow)
                    },
                    InputAction::Terminate => {
                        self.component.on_focus(&mut self.state, false);
                        event_handler(Event::Terminate, &mut self.component, &mut self.control_flow)
                    },
                    _ => self.component.on_input_action(&mut self.state, &input_action),
                }
            },
            InputEvent::Mouse(event) => self.component.on_mouse_action(
                &mut self.state,
                &self.input_method.get_mouse_action(event),
                0.0,
                row_origin,
                *screen_width as f64,
                *screen_height as f64,
            ),
            InputEvent::Resized { cols, rows } => {
                *screen_width = cols;
                *screen_height = rows;
                buf.resize(cols, rows)?;
            },
            InputEvent::Paste(clipboard) => self
                .component
                .on_input_action(&mut self.state, &InputAction::Paste(clipboard)),
            _ => (),
        }

        Ok(())
    }
}
