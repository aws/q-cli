use newton::{
    Color,
    ControlFlow,
    DisplayState,
};

use super::Label;
use crate::{
    stylable,
    BorderStyle,
    Component,
    Event,
    Style,
    StyleContext,
    StyleSheet,
};

pub struct Frame<'a> {
    component: &'a mut dyn Component,
    style: Style,
    has_title: bool,
    title_label: Label,
    title_style: Style,
}

impl<'a> Frame<'a> {
    pub const STYLE_CLASS: &'static str = "frame";

    stylable!();

    pub fn new(component: &'a mut dyn Component) -> Self {
        Self {
            component,
            style: Default::default(),
            title_label: Label::new(""),
            has_title: false,
            title_style: Default::default(),
        }
    }

    pub fn with_title(mut self, title: impl Into<String>) -> Self {
        self.has_title = true;
        self.title_label = self.title_label.with_text(title.into());
        self
    }

    pub fn with_title_border(mut self, border: BorderStyle) -> Self {
        self.title_label = self.title_label.with_border_style(border);

        self
    }

    pub fn with_title_style(mut self, style: Style) -> Self {
        self.title_style = style;
        self
    }
}

impl Component for Frame<'_> {
    fn update(
        &mut self,
        renderer: &mut DisplayState,
        style_sheet: &StyleSheet,
        control_flow: &mut ControlFlow,
        focused: bool,
        event: Event,
    ) {
        let context = StyleContext { focused, hover: false };

        match event {
            Event::Draw {
                mut x,
                mut y,
                mut width,
                mut height,
            } => {
                let style = self.style(style_sheet, context);

                let title_style =
                    style_sheet.get_style_for_component_with_class(&self.title_label, "frame.title", context);

                let offset = title_style.margin_top() + title_style.border_top_width() + title_style.padding_top();
                let y_orig = y;

                height += offset;
                style.draw_container(&mut x, &mut y, &mut width, &mut height, renderer);

                if self.has_title {
                    self.title_label.style = title_style;
                    self.title_label
                        .update(renderer, style_sheet, control_flow, focused, Event::Draw {
                            x,
                            y: y_orig - style.margin_top(),
                            width: self.title_label.desired_width(style_sheet, context),
                            height: self.title_label.desired_height(style_sheet, context),
                        });
                }

                self.component
                    .update(renderer, style_sheet, control_flow, focused, Event::Draw {
                        x,
                        y: if self.has_title {
                            y + self.title_label.style.margin_bottom()
                                + self.title_label.style.padding_bottom()
                                + self.title_label.style.border_bottom_width()
                        } else {
                            y
                        },
                        width: self.component.desired_width(style_sheet, context),
                        height: self.component.desired_height(style_sheet, context),
                    });
            },
            _ => self
                .component
                .update(renderer, style_sheet, control_flow, focused, event),
        }
    }

    fn desired_width(&self, style_sheet: &StyleSheet, context: StyleContext) -> u16 {
        self.style(style_sheet, context).spacing_horizontal() + self.component.desired_width(style_sheet, context)
    }

    fn desired_height(&self, style_sheet: &StyleSheet, context: StyleContext) -> u16 {
        self.style(style_sheet, context).spacing_vertical()
            + self.component.desired_height(style_sheet, context)
            + if self.has_title {
                self.title_label.style.border_top_width()
                    + self.title_label.style.margin_top()
                    + self.title_label.style.padding_top()
            } else {
                0
            }
    }

    fn interactive(&self) -> bool {
        self.component.interactive()
    }

    fn inline_style(&self) -> Option<Style> {
        Some(self.style)
    }

    fn class(&self) -> &str {
        "frame"
    }
}
