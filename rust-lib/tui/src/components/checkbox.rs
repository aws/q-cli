use newton::{
    Color,
    ControlFlow,
    DisplayState,
    KeyCode,
};

use crate::components::Label;
use crate::{
    stylable,
    BorderStyle,
    Component,
    Event,
    Style,
    StyleContext,
    StyleSheet,
};

#[derive(Clone, Copy, Debug)]
pub enum CheckStyle {
    Classic,
    Ascii { checked: char, unchecked: char },
}

impl Default for CheckStyle {
    fn default() -> Self {
        CheckStyle::Classic
    }
}

#[derive(Debug, Default)]
pub struct Checkbox {
    pub text: String,
    pub checked: bool,
    pub style: Style,
    pub check_style: CheckStyle,
    checkmark: Label,
    label: Label,
}

impl Checkbox {
    pub const STYLE_CLASS: &'static str = "checkbox";

    stylable!();

    pub fn new(checked: bool) -> Self {
        let text = Default::default();
        Self {
            label: Label::new(&text),
            checkmark: Label::new(""),
            checked,
            text,
            style: Default::default(),
            check_style: CheckStyle::Classic,
        }
    }

    pub fn with_text(mut self, text: impl Into<String>) -> Self {
        self.text = text.into();
        self
    }
}

impl Component for Checkbox {
    fn update(
        &mut self,
        renderer: &mut DisplayState,
        style_sheet: &StyleSheet,
        control_flow: &mut ControlFlow,
        focused: bool,
        event: Event,
    ) {
        let ctx = StyleContext { focused, hover: false };

        match event {
            Event::Draw {
                mut x,
                mut y,
                mut width,
                mut height,
            } => {
                let style = self.style(style_sheet, ctx);
                style.draw_container(&mut x, &mut y, &mut width, &mut height, renderer);

                self.checkmark.style =
                    style_sheet.get_style_for_component_with_class(&self.label, "checkbox.checkbox", ctx);
                self.label.style = style_sheet.get_style_for_component_with_class(&self.label, "checkbox.label", ctx);

                self.checkmark.label = match self.check_style {
                    CheckStyle::Classic => if self.checked { '☑' } else { '☐' }.to_string(),
                    CheckStyle::Ascii { checked, unchecked } => {
                        if self.checked { checked } else { unchecked }.to_string()
                    },
                };

                self.label.label = self.text.to_string();

                self.checkmark
                    .update(renderer, style_sheet, control_flow, focused, Event::Draw {
                        x,
                        y,
                        width: self.checkmark.desired_width(style_sheet, ctx),
                        height: self.checkmark.desired_height(style_sheet, ctx),
                    });

                self.label
                    .update(renderer, style_sheet, control_flow, focused, Event::Draw {
                        x: x + 2,
                        y,
                        width: self.label.desired_width(style_sheet, ctx),
                        height: self.label.desired_height(style_sheet, ctx),
                    });
            },
            Event::KeyPressed { code, .. } => {
                if code == KeyCode::Enter {
                    self.checked = !self.checked
                }
            },
            _ => (),
        }
    }

    fn interactive(&self) -> bool {
        true
    }

    fn desired_width(&self, style_sheet: &StyleSheet, context: StyleContext) -> u16 {
        self.style(style_sheet, context).spacing_horizontal()
            + 2
            + self.label.style(style_sheet, context).spacing_horizontal()
            + u16::try_from(self.text.len()).unwrap()
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
