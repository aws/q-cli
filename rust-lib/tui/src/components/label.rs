use std::fmt::Display;

use newton::{
    Color,
    ControlFlow,
    DisplayState,
};

use crate::{
    stylable,
    BorderStyle,
    Component,
    Event,
    Style,
    StyleContext,
    StyleSheet,
};

#[derive(Debug, Default)]
pub struct Label {
    pub label: String,
    pub style: Style,
}

impl Label {
    pub const STYLE_CLASS: &'static str = "label";

    stylable!();

    pub fn new<D: Display>(label: D) -> Self {
        let label = label.to_string();

        Self {
            label,
            style: Default::default(),
        }
    }

    pub fn with_text(mut self, text: impl Into<String>) -> Self {
        self.label = text.into();
        self
    }
}

impl Component for Label {
    fn update(
        &mut self,
        renderer: &mut DisplayState,
        style_sheet: &StyleSheet,
        _: &mut ControlFlow,
        focused: bool,
        event: Event,
    ) {
        let ctx = StyleContext { focused, hover: false };

        match event {
            Event::Initialize => {
                let len = u16::try_from(self.label.len()).unwrap();

                let style = self.style(style_sheet, ctx);

                // self.style = style
                //    .with_width(style.width.unwrap_or(len))
                //    .with_height(style.height.unwrap_or(1))
                //    .with_max_width(style.max_width.unwrap_or(128))
                //    .with_max_height(style.max_height.unwrap_or(1));
            },
            Event::Draw {
                mut x,
                mut y,
                mut width,
                mut height,
            } => {
                let style = self.style(style_sheet, ctx);
                style.draw_container(&mut x, &mut y, &mut width, &mut height, renderer);

                if height != 0 {
                    renderer.draw_string(
                        &self.label[0..self.label.len().min(usize::from(width))],
                        x,
                        y,
                        style.color(),
                        style.background_color(),
                    );
                }
            },
            _ => (),
        }
    }

    fn interactive(&self) -> bool {
        false
    }

    fn desired_width(&self, style_sheet: &StyleSheet, context: StyleContext) -> u16 {
        self.style(style_sheet, context).spacing_horizontal() + u16::try_from(self.label.len()).unwrap()
    }

    fn desired_height(&self, style_sheet: &StyleSheet, context: StyleContext) -> u16 {
        self.style(style_sheet, context).spacing_vertical() + 1
    }

    fn inline_style(&self) -> Option<Style> {
        Some(self.style)
    }

    fn class(&self) -> &str {
        Self::STYLE_CLASS
    }
}
