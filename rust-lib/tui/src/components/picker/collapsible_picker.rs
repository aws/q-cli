use newton::{
    Color,
    ControlFlow,
    DisplayState,
};

use crate::components::{
    Disclosure,
    PickerComponent,
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

pub struct CollapsiblePicker<'a, T: PickerComponent + Component> {
    style: Style,
    disclosure: Disclosure<T>,
    placeholder: &'a str,
    has_made_selection: bool,
    pub collapsed: bool,
}

impl<'a, C: PickerComponent + Component> CollapsiblePicker<'a, C> {
    pub const STYLE_CLASS: &'static str = "collapsible_picker";

    stylable!();

    pub fn new<I, T>(options: I) -> Self
    where
        I: IntoIterator<Item = T>,
        T: Into<String>,
    {
        let placeholder = "No option selected";
        Self {
            style: Default::default(),
            placeholder,
            disclosure: Disclosure::new(placeholder, C::new(options)),
            has_made_selection: false,
            collapsed: true,
        }
    }

    pub fn with_placeholder(mut self, text: &'a str) -> Self {
        self.placeholder = text;
        self
    }

    pub fn selected_index(&self) -> Option<usize> {
        if self.has_made_selection {
            self.disclosure.details.selected()
        } else {
            None
        }
    }

    pub fn options(&self) -> &Vec<String> {
        self.disclosure.details.options()
    }

    pub fn selected_item(&self) -> Option<String> {
        self.selected_index().map(|index| self.options()[index].to_string())
    }
}

impl<'a, C: PickerComponent + Component> Component for CollapsiblePicker<'a, C> {
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
                style.height = Some(self.desired_height(style_sheet, context));
                style.width = Some(self.desired_width(style_sheet, context));

                style.draw_container(&mut x, &mut y, &mut width, &mut height, renderer);

                self.disclosure.summary.label = match self.selected_item() {
                    Some(selection) => selection,
                    None => self.placeholder.to_string(),
                };

                if !self.has_made_selection && focused {
                    self.has_made_selection = true
                }

                self.disclosure
                    .update(renderer, style_sheet, control_flow, focused, Event::Draw {
                        x,
                        y,
                        width,
                        height,
                    })
            },
            _ => {
                self.disclosure.opened = focused;

                self.disclosure
                    .update(renderer, style_sheet, control_flow, focused, event);
            },
        }
    }

    fn desired_width(&self, style_sheet: &StyleSheet, context: StyleContext) -> u16 {
        self.style(style_sheet, context).spacing_horizontal() + self.disclosure.desired_width(style_sheet, context)
    }

    fn desired_height(&self, style_sheet: &StyleSheet, context: StyleContext) -> u16 {
        self.style(style_sheet, context).spacing_vertical() + self.disclosure.desired_height(style_sheet, context)
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
