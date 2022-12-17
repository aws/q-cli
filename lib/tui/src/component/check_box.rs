use std::fmt::Display;

use termwiz::surface::Surface;
use unicode_width::UnicodeWidthStr;

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

#[derive(Debug)]
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
            inner: ComponentData::new("input".to_owned(), id.to_string(), true),
        }
    }
}

impl Component for CheckBox {
    fn draw(&self, state: &mut State, surface: &mut Surface, x: f64, y: f64, width: f64, height: f64, _: f64, _: f64) {
        if width <= 0.0 || height <= 0.0 {
            return;
        }

        let style = self.style(state);

        surface.draw_text(
            &format!("{} {}", if self.checked { '☑' } else { '☐' }, self.label),
            x,
            y,
            2.0 + self.label.width() as f64,
            style.attributes(),
        );
    }

    fn on_input_action(&mut self, state: &mut State, input_action: &InputAction) {
        if let InputAction::Insert(' ') = input_action {
            self.checked = !self.checked;
            state.event_buffer.push(Event::CheckBox(CheckBoxEvent::Checked {
                id: self.inner.id.to_owned(),
                checked: self.checked,
            }))
        }
    }

    fn inner(&self) -> &ComponentData {
        &self.inner
    }

    fn inner_mut(&mut self) -> &mut ComponentData {
        &mut self.inner
    }

    fn size(&self, _: &mut State) -> (f64, f64) {
        (2.0 + self.label.width() as f64, 1.0)
    }
}

// #[cfg(test)]
// mod tests {
//     use lightningcss::stylesheet::{
//         ParserOptions,
//         StyleSheet,
//     };
//     use termwiz::input::{
//         InputEvent,
//         KeyCode,
//         KeyEvent,
//         Modifiers,
//     };
//
//     use super::*;
//     use crate::{
//         ControlFlow,
//         EventLoop,
//         InputMethod,
//     };
//
//     #[ignore = "does not work on CI"]
//     #[test]
//     fn test_checkbox() {
//         let mut test = false;
//
//         let check_box_id = "test";
//         let mut check_box = CheckBox::new(check_box_id, "Test", test);
//
//         EventLoop::new()
//             .run(
//                 &mut check_box,
//                 InputMethod::Scripted(vec![InputEvent::Key(KeyEvent {
//                     key: KeyCode::Char(' '),
//                     modifiers: Modifiers::NONE,
//                 })]),
//                 StyleSheet::parse("", ParserOptions::default()).unwrap(),
//                 |event, _component, control_flow| {
//                     if let Event::CheckBox(CheckBoxEvent::Checked { id, checked }) = event {
//                         if id == check_box_id {
//                             test = checked;
//                             *control_flow = ControlFlow::Quit
//                         }
//                     }
//                 },
//             )
//             .unwrap();
//
//         assert!(test);
//     }
// }
