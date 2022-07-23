mod check_box;
mod container;
mod label;
mod paragraph;
mod select;
mod text_field;

pub use check_box::CheckBox;
pub use container::Container;
pub use label::Label;
use newton::{
    Color,
    DisplayState,
};
pub use paragraph::Paragraph;
pub use select::Select;
pub use text_field::TextField;

use crate::input::InputAction;
use crate::{
    BorderStyle,
    Style,
    StyleSheet,
};

macro_rules! with_style_field {
    ($i:ident, $k:ident, $v:ident) => {
        pub fn $i(mut self, $k: $v) -> Self {
            self.style = self.style.$i($k);
            self
        }
    };
}

pub enum ComponentType {
    CheckBox(CheckBox),
    Container(Box<Container>),
    Label(Label),
    Paragraph(Paragraph),
    Select(Select),
    TextField(TextField),
}

pub struct Component {
    inner: ComponentType,
    style: Style,
    pub(crate) width: i32,
    pub(crate) height: i32,
    hovered: bool,
    focused: bool,
    active: bool,
}

impl Component {
    with_style_field!(with_color, color, Color);

    with_style_field!(with_background_color, background_color, Color);

    with_style_field!(with_margin_top, margin_top, i32);

    with_style_field!(with_margin_bottom, margin_bottom, i32);

    with_style_field!(with_margin_left, margin_left, i32);

    with_style_field!(with_margin_right, margin_right, i32);

    with_style_field!(with_border_top_width, border_top_width, i32);

    with_style_field!(with_border_bottom_width, border_bottom_width, i32);

    with_style_field!(with_border_left_width, border_left_width, i32);

    with_style_field!(with_border_right_width, border_right_width, i32);

    with_style_field!(with_border_style, border_style, BorderStyle);

    with_style_field!(with_border_top_color, border_top_color, Color);

    with_style_field!(with_border_bottom_color, border_bottom_color, Color);

    with_style_field!(with_border_left_color, border_left_color, Color);

    with_style_field!(with_border_right_color, border_right_color, Color);

    with_style_field!(with_height, height, i32);

    with_style_field!(with_max_height, max_height, i32);

    with_style_field!(with_max_width, max_width, i32);

    with_style_field!(with_min_height, min_height, i32);

    with_style_field!(with_min_width, min_width, i32);

    with_style_field!(with_padding_top, padding_top, i32);

    with_style_field!(with_padding_bottom, padding_bottom, i32);

    with_style_field!(with_padding_left, padding_left, i32);

    with_style_field!(with_padding_right, padding_right, i32);

    with_style_field!(with_width, width, i32);

    pub fn with_style(mut self, style: Style) -> Self {
        self.style = style;
        self
    }

    pub(crate) fn new(inner: ComponentType) -> Self {
        Self {
            inner,
            style: Style::default(),
            width: 0,
            height: 0,
            hovered: false,
            focused: false,
            active: false,
        }
    }

    pub(crate) fn initialize(&mut self, style_sheet: &StyleSheet) {
        match &mut self.inner {
            ComponentType::CheckBox(c) => c.initialize(&mut self.width, &mut self.height),
            ComponentType::Container(c) => c.initialize(style_sheet, &mut self.width, &mut self.height),
            ComponentType::Label(c) => c.initialize(&mut self.width, &mut self.height),
            ComponentType::Paragraph(c) => c.initialize(&mut self.width, &mut self.height),
            ComponentType::Select(c) => c.initialize(&mut self.width, &mut self.height),
            ComponentType::TextField(c) => c.initialize(&mut self.width, &mut self.height),
        }
    }

    #[allow(clippy::too_many_arguments)]
    pub(crate) fn draw(
        &self,
        renderer: &mut DisplayState,
        style_sheet: &StyleSheet,
        mut x: i32,
        mut y: i32,
        mut width: i32,
        mut height: i32,
        screen_width: i32,
        screen_height: i32,
    ) {
        let style = self.style(style_sheet);

        x += style.margin_left();
        y += style.margin_top();
        width -= style.margin_horizontal();
        height -= style.margin_vertical();

        match style.border_style() {
            BorderStyle::None => (),
            BorderStyle::Filled => {
                renderer.draw_rect(' ', x, y, width, height, style.color(), style.border_top_color());
                x += style.border_left_width();
                y += style.border_top_width();
                width -= style.border_horizontal();
                height -= style.border_vertical();
            },
            BorderStyle::Ascii {
                top_left,
                top,
                top_right,
                left,
                right,
                bottom_left,
                bottom,
                bottom_right,
            } => {
                renderer.draw_rect(
                    left,
                    x,
                    y,
                    style.border_left_width(),
                    height,
                    style.border_left_color(),
                    style.background_color(),
                );
                renderer.draw_rect(
                    right,
                    x + (width - style.border_right_width()),
                    y,
                    style.border_right_width(),
                    height,
                    style.border_right_color(),
                    style.background_color(),
                );
                renderer.draw_rect(
                    top,
                    x,
                    y,
                    width,
                    style.border_top_width(),
                    style.border_top_color(),
                    style.background_color(),
                );
                renderer.draw_rect(
                    bottom,
                    x,
                    y + (height - style.border_bottom_width()),
                    width,
                    style.border_bottom_width(),
                    style.border_bottom_color(),
                    style.background_color(),
                );
                renderer.draw_rect(
                    top_left,
                    x,
                    y,
                    style.border_left_width(),
                    style.border_top_width(),
                    style.border_top_color(),
                    style.background_color(),
                );
                renderer.draw_rect(
                    top_right,
                    x + (width - style.border_right_width()),
                    y,
                    style.border_right_width(),
                    style.border_top_width(),
                    style.border_top_color(),
                    style.background_color(),
                );
                renderer.draw_rect(
                    bottom_left,
                    x,
                    y + (height - style.border_bottom_width()),
                    style.border_left_width(),
                    style.border_bottom_width(),
                    style.border_bottom_color(),
                    style.background_color(),
                );
                renderer.draw_rect(
                    bottom_right,
                    x + (width - style.border_right_width()),
                    y + (height - style.border_bottom_width()),
                    style.border_right_width(),
                    style.border_bottom_width(),
                    style.border_bottom_color(),
                    style.background_color(),
                );
                x += style.border_left_width();
                y += style.border_top_width();
                width -= style.border_horizontal();
                height -= style.border_vertical();
            },
        }

        renderer.draw_rect(' ', x, y, width, height, style.color(), style.background_color());

        x += style.padding_left();
        y += style.padding_top();
        width -= style.padding_horizontal();
        height -= style.padding_vertical();

        match &self.inner {
            ComponentType::CheckBox(c) => c.draw(renderer, &style, x, y, width, height),
            ComponentType::Container(c) => c.draw(
                renderer,
                style_sheet,
                &style,
                x,
                y,
                width,
                height,
                screen_width,
                screen_height,
            ),
            ComponentType::Label(c) => c.draw(renderer, &style, x, y, width, height),
            ComponentType::Paragraph(c) => c.draw(renderer, &style, x, y, width, height),
            ComponentType::Select(c) => c.draw(renderer, &style, x, y, width, height),
            ComponentType::TextField(c) => c.draw(renderer, &style, x, y, width, height),
        }
    }

    pub(crate) fn next(&mut self, style_sheet: &StyleSheet, wrap: bool) -> Option<()> {
        if let ComponentType::Container(c) = &mut self.inner {
            return c.next(style_sheet, wrap);
        }

        None
    }

    pub(crate) fn prev(&mut self, style_sheet: &StyleSheet, wrap: bool) -> Option<()> {
        if let ComponentType::Container(c) = &mut self.inner {
            return c.prev(style_sheet, wrap);
        }

        None
    }

    pub(crate) fn interactive(&self) -> bool {
        match &self.inner {
            ComponentType::CheckBox(_) => true,
            ComponentType::Container(c) => c.interactive(),
            ComponentType::Label(_) => false,
            ComponentType::Paragraph(_) => false,
            ComponentType::Select(_) => true,
            ComponentType::TextField(_) => true,
        }
    }

    pub(crate) fn class(&self) -> &str {
        match &self.inner {
            ComponentType::CheckBox(_) => "input:checkbox",
            ComponentType::Container(_) => "div",
            ComponentType::Label(_) => "h1",
            ComponentType::Paragraph(_) => "p",
            ComponentType::Select(_) => "select",
            ComponentType::TextField(_) => "input:text",
        }
    }

    pub(crate) fn style(&self, style_sheet: &StyleSheet) -> Style {
        let style = style_sheet
            .get_style(self.class(), self.hovered, self.focused, self.active)
            .apply(&self.style);
        style.apply(&self.style)
    }

    pub(crate) fn on_input_action(&mut self, style_sheet: &StyleSheet, input_action: InputAction) {
        match &mut self.inner {
            ComponentType::CheckBox(c) => c.on_input_action(input_action),
            ComponentType::Container(c) => {
                c.on_input_action(style_sheet, &mut self.width, &mut self.height, input_action)
            },
            ComponentType::Select(c) => c.on_input_action(&mut self.height, input_action),
            ComponentType::TextField(c) => c.on_input_action(input_action),
            _ => (),
        }
    }

    pub(crate) fn on_focus(&mut self, style_sheet: &StyleSheet, focused: bool) {
        self.focused = focused;

        match &mut self.inner {
            ComponentType::Container(c) => c.on_focus(focused, style_sheet, &mut self.width, &mut self.height),
            ComponentType::Select(c) => c.on_focus(&mut self.height, focused),
            ComponentType::TextField(c) => c.on_focus(focused),
            _ => (),
        }
    }

    pub(crate) fn on_resize(&mut self, width: i32, height: i32) {
        match &mut self.inner {
            ComponentType::Container(c) => c.on_resize(width, height),
            ComponentType::Select(c) => c.on_resize(width),
            ComponentType::TextField(c) => c.on_resize(width),
            _ => (),
        }
    }
}

impl From<CheckBox> for Component {
    fn from(from: CheckBox) -> Self {
        Self::new(ComponentType::CheckBox(from))
    }
}

impl From<Container> for Component {
    fn from(from: Container) -> Self {
        Self::new(ComponentType::Container(Box::new(from)))
    }
}

impl From<Label> for Component {
    fn from(from: Label) -> Self {
        Self::new(ComponentType::Label(from))
    }
}

impl From<Paragraph> for Component {
    fn from(from: Paragraph) -> Self {
        Self::new(ComponentType::Paragraph(from))
    }
}

impl From<Select> for Component {
    fn from(from: Select) -> Self {
        Self::new(ComponentType::Select(from))
    }
}

impl From<TextField> for Component {
    fn from(from: TextField) -> Self {
        Self::new(ComponentType::TextField(from))
    }
}
