use newton::DisplayState;

use super::Component;
use crate::input::InputAction;
use crate::{
    Style,
    StyleSheet,
};

pub struct Container {
    components: Vec<Component>,
    active: Option<usize>,
}

impl Container {
    pub fn new(components: Vec<Component>) -> Self {
        Self {
            components,
            active: None,
        }
    }

    pub(crate) fn initialize(&mut self, style_sheet: &StyleSheet, width: &mut i32, height: &mut i32) {
        for component in &mut self.components {
            component.initialize(style_sheet);
        }

        self.resize(style_sheet, width, height)
    }

    #[allow(clippy::too_many_arguments)]
    pub(crate) fn draw(
        &self,
        renderer: &mut DisplayState,
        style_sheet: &StyleSheet,
        _style: &Style,
        x: i32,
        mut y: i32,
        width: i32,
        _height: i32,
        screen_width: i32,
        screen_height: i32,
    ) {
        for component in &self.components {
            let style = component.style(style_sheet);

            component.draw(
                renderer,
                style_sheet,
                x,
                y,
                (style.width().unwrap_or(component.width) + style.spacing_horizontal()).min(width),
                style.height().unwrap_or(component.height) + style.spacing_vertical(),
                screen_width,
                screen_height,
            );

            y += style.height().unwrap_or(component.height) + style.spacing_vertical();
        }
    }

    pub(crate) fn next(&mut self, style_sheet: &StyleSheet, wrap: bool) -> Option<()> {
        if let Some(active) = self.active {
            match self.components[active].next(style_sheet, false) {
                Some(_) => return Some(()),
                None => {
                    self.components[active].on_focus(style_sheet, false);
                    self.active = self
                        .components
                        .iter()
                        .enumerate()
                        .skip(active + 1)
                        .find(|(_, c)| c.interactive())
                        .map(|(i, _)| i);
                    if let Some(active) = self.active {
                        self.components[active].on_focus(style_sheet, true);
                        return Some(());
                    }
                },
            }
        }

        if self.interactive() && wrap {
            self.active = self
                .components
                .iter()
                .enumerate()
                .find(|(_, c)| c.interactive())
                .map(|(i, _)| i);

            self.components[self.active.unwrap()].on_focus(style_sheet, true);

            return Some(());
        }

        None
    }

    pub(crate) fn prev(&mut self, style_sheet: &StyleSheet, wrap: bool) -> Option<()> {
        if let Some(active) = self.active {
            match self.components[active].prev(style_sheet, false) {
                Some(_) => return Some(()),
                None => {
                    self.components[active].on_focus(style_sheet, false);
                    self.active = self.components[0..active]
                        .iter()
                        .enumerate()
                        .rev()
                        .find(|(_, c)| c.interactive())
                        .map(|(i, _)| i);
                    if let Some(active) = self.active {
                        self.components[active].on_focus(style_sheet, true);
                        return Some(());
                    }
                },
            }
        }

        if self.interactive() && wrap {
            self.active = self
                .components
                .iter()
                .enumerate()
                .rev()
                .find(|(_, c)| c.interactive())
                .map(|(i, _)| i);

            self.components[self.active.unwrap()].on_focus(style_sheet, true);

            return Some(());
        }

        None
    }

    pub(crate) fn interactive(&self) -> bool {
        self.components.iter().any(|c| c.interactive())
    }

    pub(crate) fn on_input_action(
        &mut self,
        style_sheet: &StyleSheet,
        width: &mut i32,
        height: &mut i32,
        input: InputAction,
    ) {
        if let Some(active) = self.active {
            self.components[active].on_input_action(style_sheet, input);
        }

        self.resize(style_sheet, width, height)
    }

    pub(crate) fn on_focus(&mut self, focused: bool, style_sheet: &StyleSheet, width: &mut i32, height: &mut i32) {
        if focused {
            self.active = self
                .components
                .iter()
                .enumerate()
                .find(|(_, c)| c.interactive())
                .map(|(i, _)| i);
        }

        if let Some(active) = self.active {
            self.components[active].on_focus(style_sheet, focused);
        }

        self.resize(style_sheet, width, height)
    }

    pub(crate) fn on_resize(&mut self, width: i32, height: i32) {
        for component in &mut self.components {
            component.on_resize(width, height);
        }
    }

    fn resize(&self, style_sheet: &StyleSheet, width: &mut i32, height: &mut i32) {
        let (w, h) = self.components.iter().fold((0, 0), |acc, c| {
            let style = c.style(style_sheet);
            (
                acc.0.max(style.width().unwrap_or(c.width) + style.spacing_horizontal()),
                acc.1 + style.height().unwrap_or(c.height) + style.spacing_vertical(),
            )
        });

        *width = w;
        *height = h;
    }
}
