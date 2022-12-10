mod check_box;
mod container;
mod file_picker;
mod paragraph;
mod select;
mod text_field;

pub use check_box::{
    CheckBox,
    CheckBoxEvent,
};
pub use container::{
    Container,
    Layout,
};
pub use file_picker::{
    FilePicker,
    FilePickerEvent,
};
pub use paragraph::Paragraph;
pub use select::{
    Select,
    SelectEvent,
};
use termwiz::input::{
    MouseButtons,
    MouseEvent,
};
use termwiz::surface::Surface;
pub use text_field::{
    TextField,
    TextFieldEvent,
};

use crate::event_loop::State;
use crate::input::InputAction;
use crate::Style;

#[derive(Debug, Default)]
pub struct ComponentData {
    pub id: String,
    pub width: f64,
    pub height: f64,
    pub interactive: bool,
    pub hover: bool,
    pub focus: bool,
    pub active: bool,
}

impl ComponentData {
    pub fn new(id: String, interactive: bool) -> Self {
        Self {
            id,
            interactive,
            ..Default::default()
        }
    }
}

pub trait Component: std::fmt::Debug {
    fn initialize(&mut self, state: &mut State);

    fn draw(
        &self,
        state: &mut State,
        surface: &mut Surface,
        x: f64,
        y: f64,
        width: f64,
        height: f64,
        screen_width: f64,
        screen_height: f64,
    );

    #[allow(unused_variables)]
    fn on_input_action(&mut self, state: &mut State, input_action: InputAction) -> bool {
        true
    }

    #[allow(unused_variables)]
    fn on_mouse_event(&mut self, state: &mut State, mouse_event: &MouseEvent, x: f64, y: f64, width: f64, height: f64) {
        if mouse_event.mouse_buttons.contains(MouseButtons::LEFT) {
            self.on_focus(state, true);
        } else if mouse_event.mouse_buttons.is_empty() {
            self.inner_mut().hover = true;
        }
    }

    #[allow(unused_variables)]
    fn next(&mut self, state: &mut State, wrap: bool) -> Option<String> {
        None
    }

    #[allow(unused_variables)]
    fn prev(&mut self, state: &mut State, wrap: bool) -> Option<String> {
        None
    }

    #[allow(unused_variables)]
    fn remove(&mut self, id: &str) -> Option<Box<dyn Component>> {
        None
    }

    #[allow(unused_variables)]
    fn insert(&mut self, id: &str, component: Box<dyn Component>) -> Option<Box<dyn Component>> {
        Some(component)
    }

    #[allow(unused_variables)]
    fn replace(&mut self, id: &str, component: Box<dyn Component>) -> Option<Box<dyn Component>> {
        Some(component)
    }

    #[allow(unused_variables)]
    fn on_focus(&mut self, state: &mut State, focus: bool) {
        self.inner_mut().focus = focus;

        if !focus {
            state.cursor_visibility = false;
        }
    }

    #[allow(unused_variables)]
    fn on_paste(&mut self, state: &mut State, clipboard: &str) {}

    #[allow(unused_variables)]
    fn interactive(&self, state: &mut State) -> bool {
        self.inner().interactive
    }

    fn style(&self, state: &mut State) -> Style {
        let inner = self.inner();
        state
            .style_sheet
            .get_style(self.class(), &inner.id, inner.hover, inner.focus, inner.active)
    }

    fn width(&self) -> f64 {
        self.inner().width
    }

    fn height(&self) -> f64 {
        self.inner().height
    }

    fn id(&self) -> String {
        self.inner().id.to_owned()
    }

    fn class(&self) -> &'static str;

    fn inner(&self) -> &ComponentData;

    fn inner_mut(&mut self) -> &mut ComponentData;
}
