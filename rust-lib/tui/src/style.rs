use newton::Color;

#[macro_export]
macro_rules! style {
    ( $( $prop:ident: $val:expr; )* ) => {{
        $crate::paste::paste! {
            $crate::Style::new() $( .[<with_ $prop>]($val) )*
        }
    }};
    ( ..$parent:expr; $( $prop:ident: $val:expr; )* ) => {{
        $crate::paste::paste! {
            $parent $( .[<with_ $prop>]($val) )*
        }
    }};
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

#[derive(Clone, Copy, Debug, Default)]
pub enum BorderStyle {
    #[default]
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

#[derive(Clone, Copy, Debug, Default)]
pub struct Style {
    pub background_color: Option<Color>,
    pub border_bottom_color: Option<Color>,
    pub border_bottom_width: Option<i32>,
    pub border_left_color: Option<Color>,
    pub border_left_width: Option<i32>,
    pub border_right_color: Option<Color>,
    pub border_right_width: Option<i32>,
    pub border_style: Option<BorderStyle>,
    pub border_top_color: Option<Color>,
    pub border_top_width: Option<i32>,
    pub color: Option<Color>,
    pub height: Option<i32>,
    pub margin_bottom: Option<i32>,
    pub margin_left: Option<i32>,
    pub margin_right: Option<i32>,
    pub margin_top: Option<i32>,
    pub max_height: Option<i32>,
    pub max_width: Option<i32>,
    pub min_height: Option<i32>,
    pub min_width: Option<i32>,
    pub padding_bottom: Option<i32>,
    pub padding_left: Option<i32>,
    pub padding_right: Option<i32>,
    pub padding_top: Option<i32>,
    pub width: Option<i32>,
}

impl Style {
    field!(with_background_color, background_color, Color, Color::Reset);

    field!(with_border_bottom_color, border_bottom_color, Color, Color::Reset);

    field!(with_border_bottom_width, border_bottom_width, i32, 0);

    field!(with_border_left_color, border_left_color, Color, Color::Reset);

    field!(with_border_left_width, border_left_width, i32, 0);

    field!(with_border_right_color, border_right_color, Color, Color::Reset);

    field!(with_border_right_width, border_right_width, i32, 0);

    field!(with_border_style, border_style, BorderStyle, BorderStyle::None);

    field!(with_border_top_color, border_top_color, Color, Color::Reset);

    field!(with_border_top_width, border_top_width, i32, 0);

    field!(with_color, color, Color, Color::Reset);

    field!(with_margin_bottom, margin_bottom, i32, 0);

    field!(with_margin_left, margin_left, i32, 0);

    field!(with_margin_right, margin_right, i32, 0);

    field!(with_margin_top, margin_top, i32, 0);

    field!(with_max_height, max_height, i32, 2048);

    field!(with_max_width, max_width, i32, 2048);

    field!(with_min_height, min_height, i32, 0);

    field!(with_min_width, min_width, i32, 0);

    field!(with_padding_bottom, padding_bottom, i32, 0);

    field!(with_padding_left, padding_left, i32, 0);

    field!(with_padding_right, padding_right, i32, 0);

    field!(with_padding_top, padding_top, i32, 0);

    with!(with_height, height, i32);

    with!(with_width, width, i32);

    pub fn new() -> Self {
        Self::default()
    }

    pub fn width(&self) -> Option<i32> {
        self.width
    }

    pub fn height(&self) -> Option<i32> {
        self.height
    }

    pub fn margin_horizontal(&self) -> i32 {
        self.margin_left() + self.margin_right()
    }

    pub fn margin_vertical(&self) -> i32 {
        self.margin_top() + self.margin_bottom()
    }

    pub fn border_horizontal(&self) -> i32 {
        self.border_left_width() + self.border_right_width()
    }

    pub fn border_vertical(&self) -> i32 {
        self.border_top_width() + self.border_bottom_width()
    }

    pub fn padding_horizontal(&self) -> i32 {
        self.padding_left() + self.padding_right()
    }

    pub fn padding_vertical(&self) -> i32 {
        self.padding_top() + self.padding_bottom()
    }

    pub fn spacing_top(&self) -> i32 {
        self.margin_top() + self.border_top_width() + self.padding_top()
    }

    pub fn spacing_bottom(&self) -> i32 {
        self.margin_bottom() + self.border_bottom_width() + self.padding_bottom()
    }

    pub fn spacing_left(&self) -> i32 {
        self.margin_left() + self.border_left_width() + self.padding_left()
    }

    pub fn spacing_right(&self) -> i32 {
        self.margin_right() + self.border_right_width() + self.padding_right()
    }

    pub fn spacing_vertical(&self) -> i32 {
        self.spacing_bottom() + self.spacing_top()
    }

    pub fn spacing_horizontal(&self) -> i32 {
        self.spacing_left() + self.spacing_right()
    }

    pub fn with_border_color(mut self, color: Color) -> Self {
        self.border_top_color.replace(color);
        self.border_bottom_color.replace(color);
        self.border_left_color.replace(color);
        self.border_right_color.replace(color);
        self
    }

    pub fn apply(&self, diff: &Style) -> Self {
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
}
