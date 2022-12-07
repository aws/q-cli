use std::collections::HashMap;
use std::mem::Discriminant;

use termwiz::cell::{
    CellAttributes,
    Intensity,
};
use termwiz::color::ColorAttribute;

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
    BackgroundColor(ColorAttribute),
    BorderBottomColor(ColorAttribute),
    BorderBottomWidth(f64),
    BorderLeftColor(ColorAttribute),
    BorderLeftWidth(f64),
    BorderRightColor(ColorAttribute),
    BorderRightWidth(f64),
    BorderStyle(BorderStyle),
    BorderTopColor(ColorAttribute),
    BorderTopWidth(f64),
    CaretColor(ColorAttribute),
    Color(ColorAttribute),
    FontWeight(Intensity),
    Height(Option<f64>),
    MarginBottom(f64),
    MarginLeft(f64),
    MarginRight(f64),
    MarginTop(f64),
    MaxHeight(f64),
    MaxWidth(f64),
    MinHeight(f64),
    MinWidth(f64),
    PaddingBottom(f64),
    PaddingLeft(f64),
    PaddingRight(f64),
    PaddingTop(f64),
    Width(Option<f64>),
}

#[derive(Clone, Debug, Default)]
pub struct Style(HashMap<Discriminant<Property>, Property>);

impl Style {
    field!(
        background_color,
        Property::BackgroundColor,
        ColorAttribute,
        ColorAttribute::Default
    );

    field!(
        border_bottom_color,
        Property::BorderBottomColor,
        ColorAttribute,
        ColorAttribute::Default
    );

    field!(border_bottom_width, Property::BorderBottomWidth, f64, 0.0);

    field!(
        border_left_color,
        Property::BorderLeftColor,
        ColorAttribute,
        ColorAttribute::Default
    );

    field!(border_left_width, Property::BorderLeftWidth, f64, 0.0);

    field!(
        border_right_color,
        Property::BorderRightColor,
        ColorAttribute,
        ColorAttribute::Default
    );

    field!(border_right_width, Property::BorderRightWidth, f64, 0.0);

    field!(border_style, Property::BorderStyle, BorderStyle, BorderStyle::None);

    field!(
        border_top_color,
        Property::BorderTopColor,
        ColorAttribute,
        ColorAttribute::Default
    );

    field!(border_top_width, Property::BorderTopWidth, f64, 0.0);

    field!(
        caret_color,
        Property::CaretColor,
        ColorAttribute,
        ColorAttribute::PaletteIndex(15)
    );

    field!(color, Property::Color, ColorAttribute, ColorAttribute::Default);

    field!(font_weight, Property::FontWeight, Intensity, Intensity::Normal);

    field!(margin_bottom, Property::MarginBottom, f64, 0.0);

    field!(margin_left, Property::MarginLeft, f64, 0.0);

    field!(margin_right, Property::MarginRight, f64, 0.0);

    field!(margin_top, Property::MarginTop, f64, 0.0);

    field!(max_height, Property::MaxHeight, f64, 2048.0);

    field!(max_width, Property::MaxWidth, f64, 2048.0);

    field!(min_height, Property::MinHeight, f64, 0.0);

    field!(min_width, Property::MinWidth, f64, 0.0);

    field!(padding_bottom, Property::PaddingBottom, f64, 0.0);

    field!(padding_left, Property::PaddingLeft, f64, 0.0);

    field!(padding_right, Property::PaddingRight, f64, 0.0);

    field!(padding_top, Property::PaddingTop, f64, 0.0);

    field!(height, Property::Height, Option<f64>, None);

    field!(width, Property::Width, Option<f64>, None);

    pub fn new() -> Self {
        Self::default()
    }

    pub fn margin_horizontal(&self) -> f64 {
        self.margin_left() + self.margin_right()
    }

    pub fn margin_vertical(&self) -> f64 {
        self.margin_top() + self.margin_bottom()
    }

    pub fn border_horizontal(&self) -> f64 {
        self.border_left_width() + self.border_right_width()
    }

    pub fn border_vertical(&self) -> f64 {
        self.border_top_width() + self.border_bottom_width()
    }

    pub fn padding_horizontal(&self) -> f64 {
        self.padding_left() + self.padding_right()
    }

    pub fn padding_vertical(&self) -> f64 {
        self.padding_top() + self.padding_bottom()
    }

    pub fn spacing_top(&self) -> f64 {
        self.margin_top() + self.border_top_width() + self.padding_top()
    }

    pub fn spacing_bottom(&self) -> f64 {
        self.margin_bottom() + self.border_bottom_width() + self.padding_bottom()
    }

    pub fn spacing_left(&self) -> f64 {
        self.margin_left() + self.border_left_width() + self.padding_left()
    }

    pub fn spacing_right(&self) -> f64 {
        self.margin_right() + self.border_right_width() + self.padding_right()
    }

    pub fn spacing_vertical(&self) -> f64 {
        self.spacing_bottom() + self.spacing_top()
    }

    pub fn spacing_horizontal(&self) -> f64 {
        self.spacing_left() + self.spacing_right()
    }

    pub fn with_border_width(&mut self, width: f64) -> &mut Self {
        self.with_border_left_width(width)
            .with_border_right_width(width)
            .with_border_top_width(width)
            .with_border_bottom_width(width)
    }

    pub fn with_border_color(&mut self, color: ColorAttribute) -> &mut Self {
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

    pub fn attributes(&self) -> CellAttributes {
        let foreground = self.color();
        let background = self.background_color();
        let intensity = self.font_weight();

        let mut attributes = CellAttributes::blank();
        attributes
            .set_foreground(foreground)
            .set_background(background)
            .set_underline_color(foreground)
            .set_intensity(intensity);
        attributes
    }
}
