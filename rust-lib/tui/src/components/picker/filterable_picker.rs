use newton::{
    Color,
    ControlFlow,
    DisplayState,
    KeyCode,
};

use crate::components::{
    Picker,
    PickerComponent,
    TextField,
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

#[derive(Debug, Default)]
pub struct FilterablePicker {
    style: Style,
    pub options: Vec<String>,

    input: TextField,
    picker: Picker,
    search_input_at_top: bool,
    only_show_search_input_when_focused: bool,
    focused: bool,
}

impl FilterablePicker {
    pub const STYLE_CLASS: &'static str = "picker";

    stylable!();

    pub fn with_placeholder(mut self, text: impl Into<String>) -> Self {
        self.input.hint = Some(text.into());
        self
    }

    pub fn with_search_input_at_top(mut self, at_top: bool) -> Self {
        self.search_input_at_top = at_top;
        self
    }
}

impl PickerComponent for FilterablePicker {
    fn new<I, T>(options: I) -> Self
    where
        I: IntoIterator<Item = T>,
        T: Into<String>,
    {
        let opts: Vec<String> = options.into_iter().map(|i| i.into()).collect();

        Self {
            style: Default::default(),
            input: TextField::new().with_hint("Search..."),
            options: opts.clone(),
            picker: Picker::new(opts),
            search_input_at_top: true,
            only_show_search_input_when_focused: true,
            focused: false,
        }
    }

    fn selected(&self) -> Option<usize> {
        if !self.picker.options().is_empty() {
            self.picker.selected()
        } else {
            None
        }
    }

    fn options(&self) -> &Vec<String> {
        self.picker.options()
    }

    fn set_index(&mut self, index: usize) {
        self.picker.selected = index;
    }
}

impl Component for FilterablePicker {
    fn update(
        &mut self,
        renderer: &mut DisplayState,
        style_sheet: &StyleSheet,
        control_flow: &mut ControlFlow,
        focused: bool,
        event: Event,
    ) {
        self.focused = focused;
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

                let show_search_input = !self.only_show_search_input_when_focused || self.focused;
                if self.search_input_at_top {
                    if show_search_input {
                        self.input.update(renderer, style_sheet, control_flow, focused, event);
                    }
                    self.picker
                        .update(renderer, style_sheet, control_flow, focused, Event::Draw {
                            x,
                            y: y + if show_search_input {
                                self.input.desired_height(style_sheet, context)
                            } else {
                                0
                            },
                            width,
                            height: height
                                - if show_search_input {
                                    self.input.desired_height(style_sheet, context)
                                } else {
                                    0
                                },
                        });
                } else {
                    self.picker.update(renderer, style_sheet, control_flow, focused, event);
                    if show_search_input {
                        self.input
                            .update(renderer, style_sheet, control_flow, focused, Event::Draw {
                                x,
                                y: y + self.picker.desired_height(style_sheet, context),
                                width,
                                height: height - self.picker.desired_height(style_sheet, context),
                            });
                    }
                }
            },
            Event::KeyPressed { code, .. } => {
                match code {
                    KeyCode::Up | KeyCode::Down => {
                        self.picker.update(renderer, style_sheet, control_flow, focused, event);
                    },
                    _ => {
                        self.input.update(renderer, style_sheet, control_flow, focused, event);

                        let filtered = self
                            .picker
                            .options()
                            .iter()
                            .filter(|str| str.contains(&self.input.text))
                            .cloned()
                            .collect::<Vec<String>>();

                        if !self.input.text.is_empty() {
                            self.picker.set_options(filtered);
                        } else {
                            self.picker.set_options((*self.options).to_vec());
                        }

                        // ensure selection persists after filtering
                        match (self.picker.selected(), self.picker.options().len()) {
                            (None, _) | (_, 0) => self.picker.selected = 0,
                            (Some(_index), _) => self.picker.selected = self.picker.options().len() - 1,
                        }
                    },
                }
            },
            _ => (),
        }
    }

    fn desired_width(&self, style_sheet: &StyleSheet, context: StyleContext) -> u16 {
        self.style(style_sheet, context).spacing_horizontal()
            + self
                .input
                .desired_width(style_sheet, context)
                .max(self.picker.desired_width(style_sheet, context))
    }

    fn desired_height(&self, style_sheet: &StyleSheet, context: StyleContext) -> u16 {
        let show_search_input = !self.only_show_search_input_when_focused || self.focused;

        self.style(style_sheet, context).spacing_vertical()
            + if show_search_input {
                self.input.desired_height(style_sheet, context)
            } else {
                0
            }
            + self.picker.desired_height(style_sheet, context)
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
