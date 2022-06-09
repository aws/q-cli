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
    KeyCode,
    Style,
    StyleContext,
    StyleSheet,
};

pub struct Select<'a> {
    selected: usize,
    options: &'a [&'a str],
    style: Style,
}

impl<'a> Select<'a> {
    const STYLE_CLASS: &'static str = "select";

    stylable!();

    pub fn new(options: &'a [&'a str]) -> Self {
        Self {
            selected: Default::default(),
            options,
            style: Default::default(),
        }
    }

    pub fn selected(&self) -> usize {
        self.selected
    }

    pub fn value_of_selected(&self) -> &'a str {
        self.options[self.selected]
    }
}

impl<'a> Component for Select<'a> {
    fn update(
        &mut self,
        renderer: &mut DisplayState,
        style_sheet: &StyleSheet,
        _: &mut ControlFlow,
        focused: bool,
        event: Event,
    ) {
        let context = StyleContext { focused, hover: false };

        match event {
            Event::Initialize => {
                let style = self.style(style_sheet, context);

                self.style = style
                    .with_width(
                        style.width.unwrap_or(
                            self.options
                                .iter()
                                .fold(0, |acc, option| acc.max(u16::try_from(option.len()).unwrap())),
                        ),
                    )
                    .with_height(style.height.unwrap_or(u16::try_from(self.options.len()).unwrap()))
                    .with_max_width(style.max_width.unwrap_or(128))
                    .with_max_height(
                        style
                            .max_height
                            .unwrap_or(u16::try_from(self.options.len() * 2).unwrap()),
                    );
            },
            Event::Draw {
                mut x,
                mut y,
                mut width,
                mut height,
            } => {
                let style = self.style(style_sheet, context);
                style.draw_container(&mut x, &mut y, &mut width, &mut height, renderer);

                for (i, option) in self.options.iter().enumerate() {
                    if i > usize::from(height) {
                        return;
                    }

                    let (fg, bg) = match i == self.selected {
                        true => (style.background_color(), style.color()),
                        false => (style.color(), style.background_color()),
                    };

                    renderer.draw_string(
                        &option[0..option.len().min(usize::from(width))],
                        x,
                        y + u16::try_from(i).unwrap(),
                        fg,
                        bg,
                    );
                }
            },
            Event::KeyPressed { code, .. } => {
                if focused {
                    match code {
                        KeyCode::Up => self.selected = (self.selected + self.options.len() - 1) % self.options.len(),
                        KeyCode::Down => self.selected = (self.selected + 1) % self.options.len(),
                        _ => (),
                    }
                }
            },
            _ => (),
        }
    }

    fn interactive(&self) -> bool {
        true
    }

    fn inline_style(&self) -> Option<Style> {
        Some(self.style)
    }

    fn desired_width(&self, style_sheet: &StyleSheet, context: StyleContext) -> u16 {
        self.style(style_sheet, context).spacing_horizontal()
            + self
                .options
                .iter()
                .fold(0, |acc, option| acc.max(u16::try_from(option.len()).unwrap()))
    }

    fn desired_height(&self, style_sheet: &StyleSheet, context: StyleContext) -> u16 {
        self.style(style_sheet, context).spacing_vertical() + u16::try_from(self.options.len()).unwrap()
    }

    fn class(&self) -> &str {
        Self::STYLE_CLASS
    }
}
