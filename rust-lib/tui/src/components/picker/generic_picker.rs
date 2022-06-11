use newton::{
    Color,
    ControlFlow,
    DisplayState,
};

use crate::components::{
    Label,
    PickerComponent,
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

#[derive(Debug, Default)]
pub struct Picker {
    pub selected: usize,
    options: Vec<String>,
    pub style: Style,

    rows: Vec<Label>,
}

impl Picker {
    const STYLE_CLASS: &'static str = "picker";

    stylable!();

    pub fn set_options(&mut self, options: Vec<String>) {
        self.options = options;
        self.rows = self.options.iter().map(Label::new).collect()
    }
}

impl PickerComponent for Picker {
    fn new<I, T>(options: I) -> Self
    where
        I: IntoIterator<Item = T>,
        T: Into<String>,
    {
        let opts: Vec<String> = options.into_iter().map(|i| i.into()).collect();

        Self {
            selected: Default::default(),
            options: opts.clone(),
            style: Default::default(),
            rows: opts.iter().map(Label::new).collect::<Vec<Label>>(),
        }
    }

    fn selected(&self) -> Option<usize> {
        Some(self.selected)
    }

    fn options(&self) -> &Vec<String> {
        &self.options
    }

    fn set_index(&mut self, index: usize) {
        self.selected = index;
    }
}

impl Component for Picker {
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

                let selected_style = style_sheet.get_computed_style("picker.selected", context);

                let row_style = style.apply(style_sheet.get_computed_style("picker.item", context));

                style.height = Some(self.desired_height(style_sheet, context));
                style.width = Some(self.desired_width(style_sheet, context));

                style.draw_container(&mut x, &mut y, &mut width, &mut height, renderer);

                let mut acc = 0;
                for (i, row) in self.rows.iter_mut().enumerate() {
                    row.style = if i == self.selected { selected_style } else { row_style };
                    row.update(renderer, style_sheet, control_flow, focused, Event::Draw {
                        x,
                        y: y + acc,
                        width: row.desired_width(style_sheet, context).min(width),
                        height: row.desired_height(style_sheet, context),
                    });

                    acc += row.desired_height(style_sheet, context);
                }
            },
            Event::KeyPressed { code, .. } => {
                if focused {
                    // prevent crashes if options are empty
                    if self.options.is_empty() {
                        return;
                    }

                    match code {
                        KeyCode::Up => self.selected = (self.selected + self.options.len() - 1) % self.options.len(),
                        KeyCode::Down => self.selected = (self.selected + 1) % self.options.len(),
                        _ => (),
                    }
                }
            },
            _ => {
                for (_i, component) in self.rows.iter_mut().enumerate() {
                    component.update(renderer, style_sheet, control_flow, focused, event);
                }
            },
        }
    }

    fn desired_width(&self, style_sheet: &StyleSheet, context: StyleContext) -> u16 {
        self.style(style_sheet, context).spacing_horizontal()
            + self.rows.iter().fold(0, |acc, component| {
                acc.max(component.desired_width(style_sheet, context))
            })
    }

    fn desired_height(&self, style_sheet: &StyleSheet, context: StyleContext) -> u16 {
        self.style(style_sheet, context).spacing_vertical()
            + self
                .rows
                .iter()
                .fold(0, |acc, component| acc + component.desired_height(style_sheet, context))
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
