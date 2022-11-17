use std::fmt::Display;

use termwiz::surface::Surface;

use crate::component::ComponentData;
use crate::event_loop::{
    Event,
    State,
};
use crate::input::InputAction;
use crate::surface_ext::SurfaceExt;
use crate::Component;

#[derive(Debug)]
pub enum CheckBoxEvent {
    /// The user has either checked or unchecked the box
    Checked { id: String, checked: bool },
}

pub struct CheckBox {
    label: String,
    checked: bool,
    inner: ComponentData,
}

impl CheckBox {
    pub fn new(id: impl ToString, label: impl Display, checked: bool) -> Self {
        Self {
            label: label.to_string(),
            checked,
            inner: ComponentData::new(id.to_string(), true),
        }
    }
}

impl Component for CheckBox {
    fn initialize(&mut self, _: &mut State) {
        self.inner.width = 4.0 + self.label.len() as f64;
        self.inner.height = 1.0;
    }

    fn draw(&self, state: &mut State, surface: &mut Surface, x: f64, y: f64, width: f64, height: f64, _: f64, _: f64) {
        if width <= 0.0 || height <= 0.0 {
            return;
        }

        let style = self.style(state);

        surface.draw_text(
            &format!("{} {}", if self.checked { '☑' } else { '☐' }, self.label)
                [0..(4 + self.label.len()).min(width as usize)],
            x,
            y,
            style.color(),
            style.background_color(),
            false,
        );
    }

    fn on_input_action(&mut self, state: &mut State, input_action: InputAction) -> bool {
        if let InputAction::Select = input_action {
            self.checked = !self.checked;
            state.event_buffer.push(Event::CheckBox(CheckBoxEvent::Checked {
                id: self.inner.id.to_owned(),
                checked: self.checked,
            }))
        }

        true
    }

    fn class(&self) -> &'static str {
        "input:checkbox"
    }

    fn inner(&self) -> &ComponentData {
        &self.inner
    }

    fn inner_mut(&mut self) -> &mut ComponentData {
        &mut self.inner
    }
}

#[cfg(test)]
mod tests {
    use termwiz::input::{
        InputEvent,
        KeyCode,
        KeyEvent,
        Modifiers,
    };

    use super::*;
    use crate::{
        ControlFlow,
        EventLoop,
        InputMethod,
        StyleSheet,
    };

    #[ignore = "does not work on CI"]
    #[test]
    fn test_checkbox() {
        let mut test = false;

        let check_box_id = "test";
        let mut check_box = CheckBox::new(check_box_id, "Test", test);

        EventLoop::new()
            .run(
                &mut check_box,
                InputMethod::Scripted(vec![InputEvent::Key(KeyEvent {
                    key: KeyCode::Char(' '),
                    modifiers: Modifiers::NONE,
                })]),
                StyleSheet::default(),
                |event, _component, control_flow| {
                    if let Event::CheckBox(CheckBoxEvent::Checked { id, checked }) = event {
                        if id == check_box_id {
                            test = checked;
                            *control_flow = ControlFlow::Quit
                        }
                    }
                },
            )
            .unwrap();

        assert!(test);
    }
}
