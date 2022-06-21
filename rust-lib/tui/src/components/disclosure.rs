use newton::{
    Color,
    ControlFlow,
    DisplayState,
    KeyCode,
};

use super::Label;
use crate::{
    BorderStyle,
    Component,
    Event,
    Style,
    StyleContext,
    StyleSheet,
};

#[allow(dead_code)]
#[derive(Clone, Copy, Debug)]
pub enum ChevronType {
    None,
    SolidTriangle,
    OutlinedTriangle,
    Ascii { opened: char, closed: char },
}

pub struct Disclosure<T: Component> {
    pub opened: bool,
    pub summary: Label,
    pub details: T,
    style: Style,

    chevron: ChevronType,
    unfocused_chevron: ChevronType,
}

impl<T: Component> Disclosure<T> {
    pub const STYLE_CLASS: &'static str = "disclosure";

    stylable!();

    pub fn new(summary: impl Into<String>, details: T) -> Self {
        Self {
            opened: false,
            summary: Label::new(summary.into()),
            details,
            style: Default::default(),
            chevron: ChevronType::SolidTriangle,
            unfocused_chevron: ChevronType::OutlinedTriangle,
        }
    }

    pub fn with_opened(mut self, opened: bool) -> Self {
        self.opened = opened;
        self
    }

    // pub fn with_details(mut self, details: Label) -> Self {
    //     self.details = details;
    //     self
    // }

    pub fn with_summary(mut self, summary: Label) -> Self {
        self.summary = summary;
        self
    }

    fn chevron_icon(&self, focused: bool) -> char {
        let chevron_type = if focused { self.chevron } else { self.unfocused_chevron };
        match chevron_type {
            ChevronType::None => ' ',
            ChevronType::SolidTriangle => {
                if self.opened {
                    '▾'
                } else {
                    '▸'
                }
            },
            ChevronType::OutlinedTriangle => {
                if self.opened {
                    '▿'
                } else {
                    '▹'
                }
            },
            ChevronType::Ascii { opened, closed } => {
                if self.opened {
                    opened
                } else {
                    closed
                }
            },
        }
    }
}

impl<T: Component> Component for Disclosure<T> {
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
                let mut style = self.style(style_sheet, context);

                let summary_style =
                    style_sheet.get_style_for_component_with_class(&self.summary, "disclosure.summary", context);

                let chevron_style = style.apply(style_sheet.get_computed_style("disclosure.chevron", context));

                style.width = Some(self.desired_width(style_sheet, context));
                style.height = Some(self.desired_height(style_sheet, context));
                style.draw_container(&mut x, &mut y, &mut width, &mut height, renderer);

                let chevron = self.chevron_icon(focused);
                renderer.draw_symbol(chevron, x, y, chevron_style.color(), chevron_style.background_color());
                self.summary.style = summary_style;
                self.summary
                    .update(renderer, style_sheet, control_flow, focused, Event::Draw {
                        x: x + 2,
                        y,
                        width: self.summary.desired_width(style_sheet, context).min(width - 2),
                        height: self.summary.desired_height(style_sheet, context).min(height),
                    });

                if self.opened {
                    self.details
                        .update(renderer, style_sheet, control_flow, focused, Event::Draw {
                            x: x + 2,
                            y: y + 1,
                            width: self.details.desired_width(style_sheet, context).min(width - 2),
                            height: self.details.desired_height(style_sheet, context).min(height - 1),
                        });
                }
            },
            Event::KeyPressed { code, .. } => {
                if focused {
                    match code {
                        KeyCode::Enter => {
                            if focused {
                                self.opened = !self.opened
                            }
                        },
                        KeyCode::Esc => {
                            *control_flow = ControlFlow::Exit;
                        },
                        _ => {
                            self.details
                                .update(renderer, style_sheet, control_flow, self.opened, event);
                        },
                    }
                }
            },
            _ => {
                self.details
                    .update(renderer, style_sheet, control_flow, self.opened, event);
            },
        }
    }

    fn desired_width(&self, style_sheet: &StyleSheet, context: StyleContext) -> u16 {
        self.style(style_sheet, context).spacing_horizontal()
            + 2
            + [
                self.summary.desired_width(style_sheet, context),
                self.details.desired_width(style_sheet, context),
            ]
            .iter()
            .fold(0, |acc, desired_width| acc.max(*desired_width))
    }

    fn desired_height(&self, style_sheet: &StyleSheet, context: StyleContext) -> u16 {
        self.style(style_sheet, context).spacing_vertical()
            + self.summary.desired_height(style_sheet, context)
            + if self.opened {
                self.details.desired_height(style_sheet, context)
            } else {
                0
            }
    }

    fn interactive(&self) -> bool {
        true
    }

    fn inline_style(&self) -> Option<Style> {
        Some(self.style)
    }

    fn class(&self) -> &str {
        Self::STYLE_CLASS
    }
}
