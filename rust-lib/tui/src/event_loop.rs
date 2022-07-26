use std::time::Instant;

pub use newton::{
    ControlFlow,
    DisplayMode,
    Event as NewtonEvent,
};

use crate::component::Component;
use crate::input::InputAction;
use crate::{
    InputMethod,
    StyleSheet,
};

pub struct EventLoop {
    inner_loop: newton::EventLoop,
    last_instant: Instant,
}

impl EventLoop {
    pub fn new(display_mode: DisplayMode) -> Result<Self, std::io::Error> {
        let event_loop = newton::EventLoop::new(display_mode)?;

        Ok(Self {
            inner_loop: event_loop,
            last_instant: Instant::now(),
        })
    }

    pub fn run(
        &mut self,
        component: &mut Component,
        input_method: &InputMethod,
        style_sheet: Option<&StyleSheet>,
        control_flow: ControlFlow,
    ) -> Result<ControlFlow, std::io::Error> {
        let mut width = self.inner_loop.width();
        let mut height = self.inner_loop.height();
        let default_style = StyleSheet::default();
        let style_sheet = match style_sheet {
            Some(sheet) => sheet,
            None => &default_style,
        };

        component.initialize(style_sheet);
        component.on_focus(style_sheet, true);

        self.last_instant = Instant::now();
        self.inner_loop.run(control_flow, |event, renderer, control_flow| {
            match event {
                NewtonEvent::Draw => {
                    renderer.clear();
                    component.draw(renderer, style_sheet, 0, 0, width, height, width, height);
                },
                NewtonEvent::Resized {
                    width: new_width,
                    height: new_height,
                } => {
                    width = new_width;
                    height = new_height;
                    component.on_resize(width, height);
                },
                NewtonEvent::KeyPressed { code, modifiers } => {
                    for input_action in InputAction::from_key(input_method, code, modifiers) {
                        match input_action {
                            InputAction::Submit => {
                                if component.next(style_sheet, false).is_none() {
                                    *control_flow = ControlFlow::Exit(0);
                                }
                            },
                            InputAction::Next => {
                                if component.next(style_sheet, true).is_none() {
                                    *control_flow = ControlFlow::Exit(0)
                                }
                            },
                            InputAction::Previous => {
                                component.prev(style_sheet, true);
                            },
                            InputAction::Exit(code) => *control_flow = ControlFlow::Exit(code),
                            InputAction::Reenter => {
                                *control_flow = ControlFlow::Reenter(1);
                            },
                            _ => component.on_input_action(style_sheet, input_action),
                        }
                    }
                },
                _ => (),
            }

            Ok(())
        })
    }
}
