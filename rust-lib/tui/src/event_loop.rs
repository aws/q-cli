use std::time::Instant;

use newton::{
    Color,
    KeyCode,
    KeyModifiers,
};
pub use newton::{
    ControlFlow,
    DisplayMode,
};

use crate::{
    Component,
    Event,
    StyleContext,
    StyleSheet,
};

pub struct EventLoop<'a> {
    inner_loop: newton::EventLoop,
    last_instant: Instant,
    width: u16,
    height: u16,
    style_sheet: Option<&'a StyleSheet>,
}

impl<'a> EventLoop<'a> {
    pub fn new() -> Self {
        Self {
            inner_loop: newton::EventLoop::new(),
            last_instant: Instant::now(),
            width: 0,
            height: 0,
            style_sheet: None,
        }
    }

    pub fn with_style_sheet(mut self, style_sheet: &'a StyleSheet) -> Self {
        self.style_sheet = Some(style_sheet);
        self
    }

    pub fn run<E, C>(
        &mut self,
        control_flow: ControlFlow,
        display_mode: DisplayMode,
        component: &mut C,
    ) -> Result<(), E>
    where
        C: Component,
        E: From<std::io::Error>,
    {
        let default_style = StyleSheet::default();
        let style_sheet = match self.style_sheet {
            Some(sheet) => sheet,
            None => &default_style,
        };

        self.last_instant = Instant::now();
        self.inner_loop
            .run(control_flow, display_mode, |event, renderer, control_flow| {
                let ctx = StyleContext {
                    focused: true,
                    hover: false,
                };
                match event {
                    newton::Event::Initialize { width, height } => {
                        self.width = width;
                        self.height = height;
                        component.update(renderer, style_sheet, control_flow, true, Event::Initialize)
                    },
                    newton::Event::Update => {
                        let delta_time = self.last_instant.elapsed().as_secs_f32();
                        self.last_instant = Instant::now();
                        component.update(renderer, style_sheet, control_flow, true, Event::Update { delta_time })
                    },
                    newton::Event::Draw => {
                        renderer.clear();
                        component.update(renderer, style_sheet, control_flow, true, Event::Draw {
                            x: 0,
                            y: 0,
                            width: component.desired_width(style_sheet, ctx),
                            height: component.desired_height(style_sheet, ctx),
                        })
                    },
                    newton::Event::Resized { width, height } => {
                        self.width = width;
                        self.height = height;
                    },
                    newton::Event::KeyPressed { code, modifiers } => {
                        if code == KeyCode::Esc {
                            *control_flow = ControlFlow::Exit;
                        }
                        component.update(renderer, style_sheet, control_flow, true, Event::KeyPressed {
                            code,
                            modifiers,
                        })
                    },
                    _ => (),
                }
                Ok(())
            })
    }
}
