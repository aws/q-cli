use newton::{
    Color,
    DisplayState,
};

use crate::{
    PseudoClass,
    PseudoElement,
};

#[macro_export]
macro_rules! style {
    ( $( $prop:ident: $val:expr; )* ) => {{
        $crate::paste::paste! {
            $crate::Style::new() $( .[<with_ $prop>]($val) )*
        }
    }};
}

#[macro_export]
macro_rules! __export_style_with {
    ($i:ident, $k:ident, $v:ident) => {
        pub fn $i(mut self, $k: $v) -> Self {
            self.style = self.style.$i($k);
            self
        }
    };
}

#[macro_export]
macro_rules! stylable {
    () => {
        $crate::__export_style_with!(with_color, color, Color);
        $crate::__export_style_with!(with_background_color, background_color, Color);
        $crate::__export_style_with!(with_margin_top, margin_top, u16);
        $crate::__export_style_with!(with_margin_bottom, margin_bottom, u16);
        $crate::__export_style_with!(with_margin_left, margin_left, u16);
        $crate::__export_style_with!(with_margin_right, margin_right, u16);
        $crate::__export_style_with!(with_border_top_width, border_top_width, u16);
        $crate::__export_style_with!(with_border_bottom_width, border_bottom_width, u16);
        $crate::__export_style_with!(with_border_left_width, border_left_width, u16);
        $crate::__export_style_with!(with_border_right_width, border_right_width, u16);
        $crate::__export_style_with!(with_border_style, border_style, BorderStyle);
        $crate::__export_style_with!(with_border_top_color, border_top_color, Color);
        $crate::__export_style_with!(with_border_bottom_color, border_bottom_color, Color);
        $crate::__export_style_with!(with_border_left_color, border_left_color, Color);
        $crate::__export_style_with!(with_border_right_color, border_right_color, Color);
        $crate::__export_style_with!(with_height, height, u16);
        $crate::__export_style_with!(with_max_height, max_height, u16);
        $crate::__export_style_with!(with_max_width, max_width, u16);
        $crate::__export_style_with!(with_min_height, min_height, u16);
        $crate::__export_style_with!(with_min_width, min_width, u16);
        $crate::__export_style_with!(with_padding_top, padding_top, u16);
        $crate::__export_style_with!(with_padding_bottom, padding_bottom, u16);
        $crate::__export_style_with!(with_padding_left, padding_left, u16);
        $crate::__export_style_with!(with_padding_right, padding_right, u16);
        $crate::__export_style_with!(with_width, width, u16);

        pub fn with_style(mut self, style: Style) -> Self {
            self.style = style;
            self
        }
    };
}

macro_rules! with {
    ($i:ident, $k:ident, $t:ident) => {
        pub fn $i(mut self, with: $t) -> Self {
            self.$k.replace(with);
            self
        }
    };
}

macro_rules! value_of {
    ($i:ident, $t:ident, $e:expr) => {
        pub fn $i(&self) -> $t {
            self.$i.unwrap_or($e)
        }
    };
}

macro_rules! field {
    ($i:ident, $k:ident, $t:ident, $e:expr) => {
        value_of!($k, $t, $e);
        with!($i, $k, $t);
    };
}

#[derive(Clone, Copy, Debug)]
pub enum BorderStyle {
    None,
    Filled,
    Ascii {
        top_left: char,
        top: char,
        top_right: char,
        left: char,
        right: char,
        bottom_left: char,
        bottom: char,
        bottom_right: char,
    },
}

impl Default for BorderStyle {
    fn default() -> Self {
        Self::None
    }
}

#[derive(Clone, Copy, Debug, Default)]
pub struct Style {
    pub background_color: Option<Color>,
    pub border_bottom_color: Option<Color>,
    pub border_bottom_width: Option<u16>,
    pub border_left_color: Option<Color>,
    pub border_left_width: Option<u16>,
    pub border_right_color: Option<Color>,
    pub border_right_width: Option<u16>,
    pub border_style: Option<BorderStyle>,
    pub border_top_color: Option<Color>,
    pub border_top_width: Option<u16>,
    pub color: Option<Color>,
    pub height: Option<u16>,
    pub margin_bottom: Option<u16>,
    pub margin_left: Option<u16>,
    pub margin_right: Option<u16>,
    pub margin_top: Option<u16>,
    pub max_height: Option<u16>,
    pub max_width: Option<u16>,
    pub min_height: Option<u16>,
    pub min_width: Option<u16>,
    pub padding_bottom: Option<u16>,
    pub padding_left: Option<u16>,
    pub padding_right: Option<u16>,
    pub padding_top: Option<u16>,
    pub width: Option<u16>,
}

impl Style {
    field!(with_background_color, background_color, Color, Color::Reset);

    field!(with_border_bottom_color, border_bottom_color, Color, Color::Reset);

    field!(with_border_bottom_width, border_bottom_width, u16, 0);

    field!(with_border_left_color, border_left_color, Color, Color::Reset);

    field!(with_border_left_width, border_left_width, u16, 0);

    field!(with_border_right_color, border_right_color, Color, Color::Reset);

    field!(with_border_right_width, border_right_width, u16, 0);

    field!(with_border_style, border_style, BorderStyle, BorderStyle::None);

    field!(with_border_top_color, border_top_color, Color, Color::Reset);

    field!(with_border_top_width, border_top_width, u16, 0);

    field!(with_color, color, Color, Color::White);

    field!(with_height, height, u16, 0);

    field!(with_margin_bottom, margin_bottom, u16, 0);

    field!(with_margin_left, margin_left, u16, 0);

    field!(with_margin_right, margin_right, u16, 0);

    field!(with_margin_top, margin_top, u16, 0);

    field!(with_max_height, max_height, u16, 2048);

    field!(with_max_width, max_width, u16, 2048);

    field!(with_min_height, min_height, u16, 0);

    field!(with_min_width, min_width, u16, 0);

    field!(with_padding_bottom, padding_bottom, u16, 0);

    field!(with_padding_left, padding_left, u16, 0);

    field!(with_padding_right, padding_right, u16, 0);

    field!(with_padding_top, padding_top, u16, 0);

    field!(with_width, width, u16, 0);

    pub fn new() -> Self {
        Self::default()
    }

    pub fn margin_horizontal(&self) -> u16 {
        self.margin_left() + self.margin_right()
    }

    pub fn margin_vertical(&self) -> u16 {
        self.margin_top() + self.margin_bottom()
    }

    pub fn border_horizontal(&self) -> u16 {
        self.border_left_width() + self.border_right_width()
    }

    pub fn border_vertical(&self) -> u16 {
        self.border_top_width() + self.border_bottom_width()
    }

    pub fn padding_horizontal(&self) -> u16 {
        self.padding_left() + self.padding_right()
    }

    pub fn padding_vertical(&self) -> u16 {
        self.padding_top() + self.padding_bottom()
    }

    pub fn spacing_top(&self) -> u16 {
        self.margin_top() + self.border_top_width() + self.padding_top()
    }

    pub fn spacing_bottom(&self) -> u16 {
        self.margin_bottom() + self.border_bottom_width() + self.padding_bottom()
    }

    pub fn spacing_left(&self) -> u16 {
        self.margin_left() + self.border_left_width() + self.padding_left()
    }

    pub fn spacing_right(&self) -> u16 {
        self.margin_right() + self.border_right_width() + self.padding_right()
    }

    pub fn spacing_vertical(&self) -> u16 {
        self.spacing_bottom() + self.spacing_top()
    }

    pub fn spacing_horizontal(&self) -> u16 {
        self.spacing_left() + self.spacing_right()
    }

    pub fn total_width(&self) -> u16 {
        self.spacing_horizontal() + self.width()
    }

    pub fn total_height(&self) -> u16 {
        self.spacing_vertical() + self.height()
    }

    pub fn with_border_color(mut self, color: Color) -> Self {
        self.border_top_color.replace(color);
        self.border_bottom_color.replace(color);
        self.border_left_color.replace(color);
        self.border_right_color.replace(color);
        self
    }

    pub fn apply(&self, diff: Style) -> Self {
        Self {
            background_color: diff.background_color.or(self.background_color),
            border_bottom_color: diff.border_bottom_color.or(self.border_bottom_color),
            border_bottom_width: diff.border_bottom_width.or(self.border_bottom_width),
            border_left_color: diff.border_left_color.or(self.border_left_color),
            border_left_width: diff.border_left_width.or(self.border_left_width),
            border_right_color: diff.border_right_color.or(self.border_right_color),
            border_right_width: diff.border_right_width.or(self.border_right_width),
            border_style: diff.border_style.or(self.border_style),
            border_top_color: diff.border_top_color.or(self.border_top_color),
            border_top_width: diff.border_top_width.or(self.border_top_width),
            color: diff.color.or(self.color),
            height: diff.height.or(self.height),
            margin_bottom: diff.margin_bottom.or(self.margin_bottom),
            margin_left: diff.margin_left.or(self.margin_left),
            margin_right: diff.margin_right.or(self.margin_right),
            margin_top: diff.margin_top.or(self.margin_top),
            max_height: diff.max_height.or(self.max_height),
            max_width: diff.max_width.or(self.max_width),
            min_height: diff.min_height.or(self.min_height),
            min_width: diff.min_width.or(self.min_width),
            padding_bottom: diff.padding_bottom.or(self.padding_bottom),
            padding_left: diff.padding_left.or(self.padding_left),
            padding_right: diff.padding_right.or(self.padding_right),
            padding_top: diff.padding_top.or(self.padding_top),
            width: diff.width.or(self.width),
        }
    }

    pub(crate) fn draw_container(
        &self,
        x: &mut u16,
        y: &mut u16,
        width: &mut u16,
        height: &mut u16,
        renderer: &mut DisplayState,
    ) {
        *x += self.margin_left();
        *y += self.margin_top();
        *width -= self.margin_horizontal().min(*width);
        *height -= self.margin_vertical().min(*height);

        match self.border_style() {
            BorderStyle::None => (),
            BorderStyle::Filled => {
                renderer.draw_rect(' ', *x, *y, *width, *height, self.color(), self.border_top_color());
                *x += self.border_left_width();
                *y += self.border_top_width();
                *width -= self.border_horizontal().min(*width);
                *height -= self.border_vertical().min(*height);
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
                    *x,
                    *y,
                    self.border_left_width(),
                    *height,
                    self.border_left_color(),
                    self.background_color(),
                );
                renderer.draw_rect(
                    right,
                    *x + (*width - self.border_right_width().min(*width)),
                    *y,
                    self.border_right_width(),
                    *height,
                    self.border_right_color(),
                    self.background_color(),
                );
                renderer.draw_rect(
                    top,
                    *x,
                    *y,
                    *width,
                    self.border_top_width(),
                    self.border_top_color(),
                    self.background_color(),
                );
                renderer.draw_rect(
                    bottom,
                    *x,
                    *y + (*height - self.border_bottom_width().min(*height)),
                    *width,
                    self.border_bottom_width(),
                    self.border_bottom_color(),
                    self.background_color(),
                );
                renderer.draw_rect(
                    top_left,
                    *x,
                    *y,
                    self.border_left_width(),
                    self.border_top_width(),
                    self.border_top_color(),
                    self.background_color(),
                );
                renderer.draw_rect(
                    top_right,
                    *x + (*width - self.border_right_width().min(*width)),
                    *y,
                    self.border_right_width(),
                    self.border_top_width(),
                    self.border_top_color(),
                    self.background_color(),
                );
                renderer.draw_rect(
                    bottom_left,
                    *x,
                    *y + (*height - self.border_bottom_width().min(*height)),
                    self.border_left_width(),
                    self.border_bottom_width(),
                    self.border_bottom_color(),
                    self.background_color(),
                );
                renderer.draw_rect(
                    bottom_right,
                    *x + (*width - self.border_right_width().min(*width)),
                    *y + (*height - self.border_bottom_width().min(*height)),
                    self.border_right_width(),
                    self.border_bottom_width(),
                    self.border_bottom_color(),
                    self.background_color(),
                );
                *x += self.border_left_width();
                *y += self.border_top_width();
                *width -= self.border_horizontal().min(*width);
                *height -= self.border_vertical().min(*height);
            },
        }

        renderer.draw_rect(' ', *x, *y, *width, *height, self.color(), self.background_color());

        *x += self.padding_left();
        *y += self.padding_top();
        *width -= self.padding_horizontal().min(*width);
        *height -= self.padding_vertical().min(*height);
    }

    pub(crate) fn _draw_container(
        &self,
        x: &mut u16,
        y: &mut u16,
        width: &mut u16,
        height: &mut u16,
        renderer: &mut DisplayState,
    ) {
        *width = (self.max_width() + self.padding_horizontal())
            .min(*width)
            .min(self.total_width());
        *height = (self.max_height() + self.padding_vertical())
            .min(*height)
            .min(self.total_height());

        *x += self.margin_left();
        *y += self.margin_top();
        *width -= self.margin_horizontal().min(*width);
        *height -= self.margin_vertical().min(*height);

        match self.border_style() {
            BorderStyle::None => (),
            BorderStyle::Filled => {
                renderer.draw_rect(' ', *x, *y, *width, *height, self.color(), self.border_top_color());
                *x += self.border_left_width();
                *y += self.border_top_width();
                *width -= self.border_horizontal().min(*width);
                *height -= self.border_vertical().min(*height);
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
                    *x,
                    *y,
                    self.border_left_width(),
                    *height,
                    self.border_left_color(),
                    self.background_color(),
                );
                renderer.draw_rect(
                    right,
                    *x + (*width - self.border_right_width().min(*width)),
                    *y,
                    self.border_right_width(),
                    *height,
                    self.border_right_color(),
                    self.background_color(),
                );
                renderer.draw_rect(
                    top,
                    *x,
                    *y,
                    *width,
                    self.border_top_width(),
                    self.border_top_color(),
                    self.background_color(),
                );
                renderer.draw_rect(
                    bottom,
                    *x,
                    *y + (*height - self.border_bottom_width().min(*height)),
                    *width,
                    self.border_bottom_width(),
                    self.border_bottom_color(),
                    self.background_color(),
                );
                renderer.draw_rect(
                    top_left,
                    *x,
                    *y,
                    self.border_left_width(),
                    self.border_top_width(),
                    self.border_top_color(),
                    self.background_color(),
                );
                renderer.draw_rect(
                    top_right,
                    *x + (*width - self.border_right_width().min(*width)),
                    *y,
                    self.border_right_width(),
                    self.border_top_width(),
                    self.border_top_color(),
                    self.background_color(),
                );
                renderer.draw_rect(
                    bottom_left,
                    *x,
                    *y + (*height - self.border_bottom_width().min(*height)),
                    self.border_left_width(),
                    self.border_bottom_width(),
                    self.border_bottom_color(),
                    self.background_color(),
                );
                renderer.draw_rect(
                    bottom_right,
                    *x + (*width - self.border_right_width().min(*width)),
                    *y + (*height - self.border_bottom_width().min(*height)),
                    self.border_right_width(),
                    self.border_bottom_width(),
                    self.border_bottom_color(),
                    self.background_color(),
                );
                *x += self.border_left_width();
                *y += self.border_top_width();
                *width -= self.border_horizontal().min(*width);
                *height -= self.border_vertical().min(*height);
            },
        }

        renderer.draw_rect(' ', *x, *y, *width, *height, self.color(), self.background_color());

        *x += self.padding_left();
        *y += self.padding_top();
        *width -= self.padding_horizontal().min(*width);
        *height -= self.padding_vertical().min(*height);
    }

    pub fn selector_for(
        element: &str,
        class: Option<&str>,
        pseudo_class: Option<PseudoClass>,
        pseudo_element: Option<PseudoElement>,
    ) -> String {
        let mut selector: String = String::new();

        selector.push_str(element);

        match class {
            Some(class) => selector.push_str(class),
            None => (),
        }

        match pseudo_class {
            Some(pseudo_class) => selector.push_str(pseudo_class.to_string().as_str()),
            None => (),
        }

        match pseudo_element {
            Some(pseudo_element) => selector.push_str(pseudo_element.to_string().as_str()),
            None => (),
        }

        selector
    }
}

#[derive(Clone, Copy, Debug, Default)]
pub struct StyleContext {
    pub focused: bool,
    pub hover: bool,
}
