use newton::DisplayState;

use crate::{
    ControlFlow,
    Event,
    Style,
    StyleContext,
    StyleSheet,
};

pub trait Component {
    fn update(
        &mut self,
        renderer: &mut DisplayState,
        style_sheet: &StyleSheet,
        control_flow: &mut ControlFlow,
        focused: bool,
        event: Event,
    );

    fn interactive(&self) -> bool;
    fn class(&self) -> &str;

    fn inline_style(&self) -> Option<Style> {
        None
    }

    fn style(&self, style_sheet: &StyleSheet, context: StyleContext) -> Style {
        style_sheet.get_style_for_element(self.class(), self.inline_style(), context)
    }

    fn desired_width(&self, style_sheet: &StyleSheet, context: StyleContext) -> u16;
    // {
    //     self.style(style_sheet, context).total_width()
    // }

    fn desired_height(&self, style_sheet: &StyleSheet, context: StyleContext) -> u16;
    // {
    //     self.style(style_sheet, context).total_height()
    // }
}
