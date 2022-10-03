use std::collections::HashMap;
use std::mem::Discriminant;

use newton::Color;

#[macro_export]
macro_rules! style {
    ( $( $prop:ident: $val:expr; )* ) => {{
        $crate::paste::paste! {
            let mut style = $crate::Style::new();
            style $( .[<with_ $prop>]($val) )*;
            style
        }
    }};
    ( ..$parent:expr; $( $prop:ident: $val:expr; )* ) => {{
        $crate::paste::paste! {
            $parent $( .[<with_ $prop>]($val) )*
        }
    }};
}

macro_rules! field {
    ($i:ident, $k:path, $t:ty, $e:expr) => {
        pub fn $i(&self) -> $t {
            let property = $k(unsafe { std::mem::zeroed() });
            if let $k(val) = self.0.get(&std::mem::discriminant(&property)).unwrap_or(&$k($e)) {
                *val
            } else {
                panic!("style property mismatch");
            }
        }

        $crate::paste::paste! {
            pub fn [<with_ $i>](&mut self, with: $t) -> &mut Self {
                let property = $k(with);
                self.0.insert(std::mem::discriminant(&property), property);
                self
            }
        }
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

#[derive(Clone, Copy, Debug)]
#[non_exhaustive]
pub enum Property {
    BackgroundColor(Color),
    BorderBottomColor(Color),
    BorderBottomWidth(i32),
    BorderLeftColor(Color),
    BorderLeftWidth(i32),
    BorderRightColor(Color),
    BorderRightWidth(i32),
    BorderStyle(BorderStyle),
    BorderTopColor(Color),
    BorderTopWidth(i32),
    Color(Color),
    Height(Option<i32>),
    MarginBottom(i32),
    MarginLeft(i32),
    MarginRight(i32),
    MarginTop(i32),
    MaxHeight(i32),
    MaxWidth(i32),
    MinHeight(i32),
    MinWidth(i32),
    PaddingBottom(i32),
    PaddingLeft(i32),
    PaddingRight(i32),
    PaddingTop(i32),
    Width(Option<i32>),
}

#[derive(Clone, Debug, Default)]
pub struct Style(HashMap<Discriminant<Property>, Property>);

impl Style {
    field!(background_color, Property::BackgroundColor, Color, Color::Reset);

    field!(border_bottom_color, Property::BorderBottomColor, Color, Color::Reset);

    field!(border_bottom_width, Property::BorderBottomWidth, i32, 0);

    field!(border_left_color, Property::BorderLeftColor, Color, Color::Reset);

    field!(border_left_width, Property::BorderLeftWidth, i32, 0);

    field!(border_right_color, Property::BorderRightColor, Color, Color::Reset);

    field!(border_right_width, Property::BorderRightWidth, i32, 0);

    field!(border_style, Property::BorderStyle, BorderStyle, BorderStyle::None);

    field!(border_top_color, Property::BorderTopColor, Color, Color::Reset);

    field!(border_top_width, Property::BorderTopWidth, i32, 0);

    field!(color, Property::Color, Color, Color::Reset);

    field!(margin_bottom, Property::MarginBottom, i32, 0);

    field!(margin_left, Property::MarginLeft, i32, 0);

    field!(margin_right, Property::MarginRight, i32, 0);

    field!(margin_top, Property::MarginTop, i32, 0);

    field!(max_height, Property::MaxHeight, i32, 2048);

    field!(max_width, Property::MaxWidth, i32, 2048);

    field!(min_height, Property::MinHeight, i32, 0);

    field!(min_width, Property::MinWidth, i32, 0);

    field!(padding_bottom, Property::PaddingBottom, i32, 0);

    field!(padding_left, Property::PaddingLeft, i32, 0);

    field!(padding_right, Property::PaddingRight, i32, 0);

    field!(padding_top, Property::PaddingTop, i32, 0);

    field!(height, Property::Height, Option<i32>, None);

    field!(width, Property::Width, Option<i32>, None);

    pub fn new() -> Self {
        Self::default()
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

    pub fn with_border_color(&mut self, color: Color) -> &mut Self {
        self.with_border_top_color(color)
            .with_border_bottom_color(color)
            .with_border_left_color(color)
            .with_border_right_color(color)
    }

    pub fn apply(&mut self, diff: &Style) -> &mut Self {
        for (key, value) in &diff.0 {
            self.0.insert(*key, *value);
        }
        self
    }

    pub fn applied(&self, diff: &Style) -> Self {
        let mut style = self.clone();
        for (key, value) in &diff.0 {
            style.0.insert(*key, *value);
        }
        style
    }
}
