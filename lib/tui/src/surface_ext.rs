use termwiz::cell::{
    AttributeChange,
    Intensity,
};
use termwiz::color::ColorAttribute;
use termwiz::surface::{
    Change,
    Position,
    Surface,
};

use crate::{
    BorderStyle,
    Style,
};

pub trait SurfaceExt {
    fn draw_text(
        &mut self,
        text: impl ToString,
        x: f64,
        y: f64,
        color: ColorAttribute,
        background_color: ColorAttribute,
        bold: bool,
    );

    fn draw_rect(
        &mut self,
        symbol: char,
        x: f64,
        y: f64,
        width: f64,
        height: f64,
        color: ColorAttribute,
        background_color: ColorAttribute,
    );

    fn draw_border(&mut self, style: &Style, x: &mut f64, y: &mut f64, width: &mut f64, height: &mut f64);
}

impl SurfaceExt for Surface {
    fn draw_text(
        &mut self,
        text: impl ToString,
        x: f64,
        y: f64,
        color: ColorAttribute,
        background_color: ColorAttribute,
        bold: bool,
    ) {
        if x < 0.0 || y < 0.0 {
            return;
        }

        let intensity = match bold {
            true => Intensity::Bold,
            false => Intensity::Normal,
        };

        self.add_changes(vec![
            Change::CursorPosition {
                x: Position::Absolute(x.round() as usize),
                y: Position::Absolute(y.round() as usize),
            },
            Change::Attribute(AttributeChange::Foreground(color)),
            Change::Attribute(AttributeChange::Background(background_color)),
            Change::Attribute(AttributeChange::Intensity(intensity)),
            Change::Text(text.to_string()),
        ]);
    }

    fn draw_rect(
        &mut self,
        symbol: char,
        x: f64,
        y: f64,
        width: f64,
        height: f64,
        color: ColorAttribute,
        background_color: ColorAttribute,
    ) {
        let x = x.round().max(0.0);
        let y = y.round().max(0.0);
        let width = width.round();
        let height = height.round();

        if width <= 0.0 || height <= 0.0 {
            return;
        }

        let x = x as usize;
        let y = y as usize;
        let width = width as usize;
        let height = height as usize;

        let text: String = vec![symbol; width].iter().collect();

        self.add_changes(vec![
            Change::Attribute(AttributeChange::Foreground(color)),
            Change::Attribute(AttributeChange::Background(background_color)),
        ]);
        for row in 0..height {
            self.add_changes(vec![
                Change::CursorPosition {
                    x: Position::Absolute(x),
                    y: Position::Absolute(y + row),
                },
                Change::Text(text.clone()),
            ]);
        }
    }

    fn draw_border(&mut self, style: &Style, x: &mut f64, y: &mut f64, width: &mut f64, height: &mut f64) {
        *x += style.margin_left();
        *y += style.margin_top();
        *width -= style.margin_horizontal();
        *height -= style.margin_vertical();

        match style.border_style() {
            BorderStyle::None => (),
            BorderStyle::Filled => {
                self.draw_rect(' ', *x, *y, *width, *height, style.color(), style.border_top_color());
                *x += style.border_left_width();
                *y += style.border_top_width();
                *width -= style.border_horizontal();
                *height -= style.border_vertical();
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
                self.draw_rect(
                    left,
                    *x,
                    *y,
                    style.border_left_width(),
                    *height,
                    style.border_left_color(),
                    style.background_color(),
                );
                self.draw_rect(
                    right,
                    *x + (*width - style.border_right_width()),
                    *y,
                    style.border_right_width(),
                    *height,
                    style.border_right_color(),
                    style.background_color(),
                );
                self.draw_rect(
                    top,
                    *x,
                    *y,
                    *width,
                    style.border_top_width(),
                    style.border_top_color(),
                    style.background_color(),
                );
                self.draw_rect(
                    bottom,
                    *x,
                    *y + (*height - style.border_bottom_width()),
                    *width,
                    style.border_bottom_width(),
                    style.border_bottom_color(),
                    style.background_color(),
                );
                self.draw_rect(
                    top_left,
                    *x,
                    *y,
                    style.border_left_width(),
                    style.border_top_width(),
                    style.border_top_color(),
                    style.background_color(),
                );
                self.draw_rect(
                    top_right,
                    *x + (*width - style.border_right_width()),
                    *y,
                    style.border_right_width(),
                    style.border_top_width(),
                    style.border_top_color(),
                    style.background_color(),
                );
                self.draw_rect(
                    bottom_left,
                    *x,
                    *y + (*height - style.border_bottom_width()),
                    style.border_left_width(),
                    style.border_bottom_width(),
                    style.border_bottom_color(),
                    style.background_color(),
                );
                self.draw_rect(
                    bottom_right,
                    *x + (*width - style.border_right_width()),
                    *y + (*height - style.border_bottom_width()),
                    style.border_right_width(),
                    style.border_bottom_width(),
                    style.border_bottom_color(),
                    style.background_color(),
                );
                *x += style.border_left_width();
                *y += style.border_top_width();
                *width -= style.border_horizontal();
                *height -= style.border_vertical();
            },
        }

        self.draw_rect(' ', *x, *y, *width, *height, style.color(), style.background_color());

        *x += style.padding_left();
        *y += style.padding_top();
        *width -= style.padding_horizontal();
        *height -= style.padding_vertical();
    }
}
