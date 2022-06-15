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
    StyleSheet,
};

pub struct Container<'a, const N: usize> {
    cursor: usize,
    components: [&'a mut dyn Component; N],
    style: Style,
}

impl<'a, const N: usize> Container<'a, N> {
    pub const STYLE_CLASS: &'static str = "table";

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
            },
            Event::Draw {
                mut x,
                mut y,
                mut width,
                mut height,
            } => {
                let style = style_sheet
                    .get_style("*")
                    .apply(style_sheet.get_style(Self::STYLE_CLASS))
                    .apply(self.style);

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
                            height: height - acc.min(height),
                        },
                    );
                    acc += component.desired_height(style_sheet);
                }
            },
            Event::KeyPressed { code, .. } => {
                if self.interactive() {
                    match code {
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

    fn desired_width(&self, style_sheet: &StyleSheet) -> u16 {
        let style = style_sheet
            .get_style("*")
            .apply(style_sheet.get_style(Self::STYLE_CLASS))
            .apply(self.style);

        style.spacing_horizontal()
            + self
                .components
                .iter()
                .fold(0, |acc, component| acc.max(component.desired_width(style_sheet)))
    }

    fn desired_height(&self, style_sheet: &StyleSheet) -> u16 {
        let style = style_sheet
            .get_style("*")
            .apply(style_sheet.get_style(Self::STYLE_CLASS))
            .apply(self.style);

        style.spacing_vertical()
            + self
                .components
                .iter()
                .fold(0, |acc, component| acc + component.desired_height(style_sheet))
    }

    fn interactive(&self) -> bool {
        self.components.iter().any(|component| component.interactive())
    }
}
