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

pub struct Form<'a> {
    cursor: usize,
    components: Vec<&'a mut dyn Component>,
    style: Style,
}

impl<'a> Form<'a> {
    pub const STYLE_CLASS: &'static str = "div";

    stylable!();

    pub fn new(components: Vec<&'a mut dyn Component>) -> Self {
        Self {
            cursor: Default::default(),
            components,
            style: Default::default(),
        }
    }
}

impl Component for Form<'_> {
    fn update(
        &mut self,
        renderer: &mut DisplayState,
        style_sheet: &StyleSheet,
        control_flow: &mut ControlFlow,
        focused: bool,
        event: Event,
    ) {
        let ctx = StyleContext {
            focused: true,
            hover: false,
        };
        match event {
            Event::Initialize => {
                for (i, component) in self.components.iter_mut().enumerate() {
                    component.update(
                        renderer,
                        style_sheet,
                        control_flow,
                        focused && (self.cursor == i),
                        event,
                    );
                }

                while self.interactive() {
                    match self.components[self.cursor].interactive() {
                        true => break,
                        false => self.cursor += 1,
                    }
                }

                let style = self.style(style_sheet, ctx);

                self.style = style
                    .with_width(style.width.unwrap_or_else(|| {
                        self.components
                            .iter()
                            .fold(0, |acc, component| acc.max(component.desired_width(style_sheet, ctx)))
                    }))
                    .with_height(style.height.unwrap_or_else(|| {
                        self.components
                            .iter()
                            .fold(0, |acc, component| acc + component.desired_height(style_sheet, ctx))
                    }))
                    .with_max_width(1024)
                    .with_max_height(1024);
            },
            Event::Draw {
                mut x,
                mut y,
                mut width,
                mut height,
            } => {
                let style = self.style(style_sheet, ctx);

                style.draw_container(&mut x, &mut y, &mut width, &mut height, renderer);

                let mut acc = 0;
                for (i, component) in self.components.iter_mut().enumerate() {
                    component.update(
                        renderer,
                        style_sheet,
                        control_flow,
                        focused && (self.cursor == i),
                        Event::Draw {
                            x,
                            y: y + acc,
                            width,
                            height: component.desired_height(style_sheet, ctx),
                        },
                    );
                    acc += component.desired_height(style_sheet, ctx);
                }
            },
            Event::KeyPressed { code, .. } => {
                if self.interactive() {
                    match code {
                        newton::KeyCode::Esc => *control_flow = ControlFlow::Return(1),
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
                        newton::KeyCode::Enter => loop {
                            let cursor = self.cursor + 1;
                            match self.components.get(cursor) {
                                Some(component) => {
                                    self.cursor = cursor;
                                    if component.interactive() {
                                        break;
                                    }
                                },
                                None => {
                                    *control_flow = ControlFlow::Exit;
                                    break;
                                },
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
                        newton::KeyCode::Down => {
                            match self.components.get(self.cursor) {
                                Some(component) => {
                                    let cursor = (self.cursor + 1) % self.components.len();

                                    #[allow(clippy::single_match)]
                                    match component.class() {
                                        "textfield" => {
                                            self.cursor = cursor;
                                            return;
                                        },
                                        _ => (),
                                    }
                                },
                                None => (),
                            }

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
                        newton::KeyCode::Up => {
                            match self.components.get(self.cursor) {
                                Some(component) => {
                                    let cursor = (self.cursor + self.components.len() - 1) % self.components.len();

                                    #[allow(clippy::single_match)]
                                    match component.class() {
                                        "textfield" => {
                                            self.cursor = cursor;
                                            return;
                                        },
                                        _ => (),
                                    }
                                },
                                None => (),
                            }

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

    fn inline_style(&self) -> Option<Style> {
        Some(self.style)
    }

    fn class(&self) -> &str {
        Self::STYLE_CLASS
    }
}
