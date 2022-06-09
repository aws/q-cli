use newton::{
    Color,
    DisplayState,
};

use crate::{
    stylable,
    BorderStyle,
    Component,
    ControlFlow,
    Event,
    Style,
    StyleContext,
    StyleSheet,
};

pub struct Container<'a, const N: usize> {
    cursor: usize,
    components: [&'a mut dyn Component; N],
    style: Style,
}

impl<'a, const N: usize> Container<'a, N> {
    pub const STYLE_CLASS: &'static str = "div";

    stylable!();

    pub fn new(components: [&'a mut dyn Component; N]) -> Self {
        Self {
            cursor: Default::default(),
            components,
            style: Default::default(),
        }
    }
}

impl<const N: usize> Component for Container<'_, N> {
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
            Event::Initialize => {
                let mut cursor = 0;
                while self.components.iter().any(|component| component.interactive()) {
                    match self.components[cursor].interactive() {
                        true => break,
                        false => cursor += 1,
                    }
                }

                let style = self.style(style_sheet, ctx);

                self.style = style
                    .with_width(style.width.unwrap_or(self.components.iter().fold(0, |acc, component| {
                        acc.max(component.style(style_sheet, ctx).total_width())
                    })))
                    .with_height(style.height.unwrap_or(self.components.iter().fold(0, |acc, component| {
                        acc + component.style(style_sheet, ctx).total_height()
                    })))
                    .with_max_width(style.max_width.unwrap_or(1024))
                    .with_max_height(style.max_height.unwrap_or(1024));
            },
            Event::Draw {
                mut x,
                mut y,
                mut width,
                mut height,
            } => {
                if width == 0 || height == 0 {
                    return;
                }

                let mut style = self.style(style_sheet, ctx);
                // style.height = Some(self.desired_height(style_sheet, ctx));
                // style.width = Some(self.desired_width(style_sheet, ctx));
                style.draw_container(&mut x, &mut y, &mut width, &mut height, renderer);

                let mut row = 0;
                for (i, component) in self.components.iter_mut().enumerate() {
                    component.update(
                        renderer,
                        style_sheet,
                        control_flow,
                        focused && (self.cursor == i),
                        Event::Draw {
                            x,
                            y: y + row,
                            width,
                            height: component.desired_height(style_sheet, ctx).min(height),
                        },
                    );
                    row += component.desired_width(style_sheet, ctx);
                    height -= component.desired_height(style_sheet, ctx).min(height);
                }
            },
            Event::KeyPressed { code, .. } => {
                if self.interactive() {
                    match code {
                        newton::KeyCode::Esc => *control_flow = ControlFlow::Exit,
                        newton::KeyCode::Tab => loop {
                            let cursor = (self.cursor + 1) % self.components.len();
                            match self.components.get(cursor) {
                                Some(component) => {
                                    self.cursor = cursor;
                                    if component.interactive() {
                                        break;
                                    }
                                },
                                None => break,
                            }
                        },
                        newton::KeyCode::BackTab => loop {
                            let cursor = (self.cursor + self.components.len() - 1) % self.components.len();
                            match self.components.get(cursor) {
                                Some(component) => {
                                    self.cursor = cursor;
                                    if component.interactive() {
                                        break;
                                    }
                                },
                                None => break,
                            }
                        },
                        _ => {
                            for (i, component) in self.components.iter_mut().enumerate() {
                                component.update(
                                    renderer,
                                    style_sheet,
                                    control_flow,
                                    focused && (self.cursor == i),
                                    event,
                                );
                            }
                        },
                    }
                }
            },
            _ => {
                for (i, component) in self.components.iter_mut().enumerate() {
                    component.update(
                        renderer,
                        style_sheet,
                        control_flow,
                        focused && (self.cursor == i),
                        event,
                    );
                }
            },
        }
    }

    fn interactive(&self) -> bool {
        self.components.iter().any(|component| component.interactive())
    }

    fn desired_width(&self, style_sheet: &StyleSheet, context: StyleContext) -> u16 {
        self.style(style_sheet, context).spacing_horizontal()
            + self.components.iter().fold(0, |acc, component| {
                acc.max(component.desired_width(style_sheet, context))
            })
    }

    fn desired_height(&self, style_sheet: &StyleSheet, context: StyleContext) -> u16 {
        self.style(style_sheet, context).spacing_vertical()
            + self
                .components
                .iter()
                .fold(0, |acc, component| acc + component.desired_height(style_sheet, context))
    }

    fn class(&self) -> &str {
        Self::STYLE_CLASS
    }

    fn inline_style(&self) -> Option<Style> {
        Some(self.style)
    }
}
