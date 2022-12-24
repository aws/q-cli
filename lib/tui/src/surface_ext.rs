use termwiz::cell::CellAttributes;
use termwiz::surface::{
    Change,
    Position,
    Surface,
};
use unicode_segmentation::UnicodeSegmentation;
use unicode_width::UnicodeWidthStr;

use crate::{
    BorderStyle,
    Style,
};

pub trait SurfaceExt {
    fn draw_text(&mut self, text: impl ToString, x: f64, y: f64, width: f64, attributes: CellAttributes);

    fn draw_rect(&mut self, symbol: char, x: f64, y: f64, width: f64, height: f64, attributes: CellAttributes);

    fn draw_border(&mut self, x: f64, y: f64, width: f64, height: f64, style: &Style);
}

impl SurfaceExt for Surface {
    fn draw_text(&mut self, text: impl ToString, x: f64, y: f64, width: f64, attributes: CellAttributes) {
        if x < 0.0 || y < 0.0 || width <= 0.0 {
            return;
        }

        let width = width.round() as usize;

        let mut drawn = String::new();
        let mut drawn_width = 0;
        for grapheme in text.to_string().graphemes(true) {
            let grapheme_width = grapheme.width();
            if drawn_width + grapheme_width <= width {
                drawn_width += grapheme_width;
                drawn.push_str(grapheme);
            } else {
                break;
            }
        }

        self.add_changes(vec![
            Change::CursorPosition {
                x: Position::Absolute(x.round() as usize),
                y: Position::Absolute(y.round() as usize),
            },
            Change::AllAttributes(attributes),
            Change::Text(drawn),
        ]);
    }

    fn draw_rect(&mut self, symbol: char, x: f64, y: f64, width: f64, height: f64, attributes: CellAttributes) {
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

        self.add_change(Change::AllAttributes(attributes));
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

    fn draw_border(&mut self, mut x: f64, mut y: f64, mut width: f64, mut height: f64, style: &Style) {
        match style.border_style() {
            BorderStyle::None => (),
            BorderStyle::Filled => {
                let mut attributes = CellAttributes::blank();
                attributes.set_foreground(style.color());
                attributes.set_background(style.border_top_color());
                self.draw_rect(' ', x, y, width, height, attributes);
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
                let mut attributes = CellAttributes::blank();
                attributes.set_foreground(style.border_left_color());
                attributes.set_background(style.background_color());

                self.draw_rect(left, x, y, style.border_left_width(), height, attributes.clone());

                attributes.set_foreground(style.border_right_color());
                self.draw_rect(
                    right,
                    x + (width - style.border_right_width()),
                    y,
                    style.border_right_width(),
                    height,
                    attributes.clone(),
                );

                attributes.set_foreground(style.border_top_color());
                self.draw_rect(top, x, y, width, style.border_top_width(), attributes.clone());

                attributes.set_foreground(style.border_bottom_color());
                self.draw_rect(
                    bottom,
                    x,
                    y + (height - style.border_bottom_width()),
                    width,
                    style.border_bottom_width(),
                    attributes.clone(),
                );

                attributes.set_foreground(style.border_top_color());
                self.draw_rect(
                    top_left,
                    x,
                    y,
                    style.border_left_width(),
                    style.border_top_width(),
                    attributes.clone(),
                );

                attributes.set_foreground(style.border_top_color());
                self.draw_rect(
                    top_right,
                    x + (width - style.border_right_width()),
                    y,
                    style.border_right_width(),
                    style.border_top_width(),
                    attributes.clone(),
                );

                attributes.set_foreground(style.border_bottom_color());
                self.draw_rect(
                    bottom_left,
                    x,
                    y + (height - style.border_bottom_width()),
                    style.border_left_width(),
                    style.border_bottom_width(),
                    attributes.clone(),
                );

                attributes.set_foreground(style.border_bottom_color());
                self.draw_rect(
                    bottom_right,
                    x + (width - style.border_right_width()),
                    y + (height - style.border_bottom_width()),
                    style.border_right_width(),
                    style.border_bottom_width(),
                    attributes,
                );
            },
        }

        x += style.border_left_width();
        y += style.border_top_width();
        width -= style.border_horizontal();
        height -= style.border_vertical();

        self.draw_rect(' ', x, y, width, height, style.attributes());
    }
}
