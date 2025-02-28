use std::io::Write;

use crossterm::{
    cursor,
    queue,
    style,
};
use eyre::Result;
use unicode_width::UnicodeWidthStr;

const TOP_LEFT_CORNER: &str = "┌";
const TOP_RIGHT_CORNER: &str = "┐";
const BOT_LEFT_CORNER: &str = "└";
const BOT_RIGHT_CORNER: &str = "┘";
const HOR_LINE: &str = "─";
const VER_LINE: &str = "│";

pub struct DialogBox<'a> {
    title: &'a str,
    text_to_display: &'a str,
    padding: usize,
}

impl<'a> DialogBox<'a> {
    pub fn new(title: &'a str, text_to_display: &'a str, padding: usize) -> Self {
        Self {
            title,
            text_to_display,
            padding,
        }
    }

    pub fn queue_boxed_output<W: Write>(&self, output: &mut W) -> Result<()> {
        let stripped_text = String::from_utf8(strip_ansi_escapes::strip(self.text_to_display))
            .map_err(|e| eyre::eyre!("failed to convert tool content to string: {:?}", e))?;
        let longest_line_width = &stripped_text.lines().fold(0_usize, |acc, line| {
            let grapheme_width = UnicodeWidthStr::width_cjk(line.trim());
            if acc > grapheme_width { acc } else { grapheme_width }
        });
        let top_right_corner_width = UnicodeWidthStr::width_cjk(TOP_RIGHT_CORNER);
        let bot_left_corner_width = UnicodeWidthStr::width_cjk(BOT_LEFT_CORNER);
        let bot_right_corner_width = UnicodeWidthStr::width_cjk(BOT_RIGHT_CORNER);
        let box_width = longest_line_width
            + UnicodeWidthStr::width_cjk(" ") * self.padding * 2
            + UnicodeWidthStr::width_cjk(VER_LINE) * 2;
        let mut boxed_content = TOP_LEFT_CORNER.to_string();
        boxed_content.push_str({
            let length = box_width - bot_left_corner_width - bot_right_corner_width;
            &HOR_LINE.repeat(length)
        });
        boxed_content.push_str(TOP_RIGHT_CORNER);
        boxed_content.push('\n');
        for line in self.text_to_display.lines() {
            let mut new_line = VER_LINE.to_string();
            new_line.push_str(&" ".repeat(self.padding));
            new_line.push_str(line.trim());
            new_line.push_str(&" ".repeat({
                let new_line_stripped = String::from_utf8(strip_ansi_escapes::strip(&new_line))
                    .map_err(|e| eyre::eyre!("failed to convert tool content to string: {:?}", e))?;
                let cur_width = UnicodeWidthStr::width_cjk(new_line_stripped.as_str());
                box_width - cur_width - top_right_corner_width
            }));
            new_line.push_str(VER_LINE);
            new_line.push('\n');
            boxed_content.push_str(&new_line);
        }
        boxed_content.push_str(BOT_LEFT_CORNER);
        boxed_content.push_str({
            let length = box_width - bot_left_corner_width - bot_right_corner_width;
            &HOR_LINE.repeat(length)
        });
        boxed_content.push_str(BOT_RIGHT_CORNER);
        boxed_content.push('\n');

        let box_height: u16 = boxed_content.split('\n').count().try_into()?;
        Ok(queue!(
            output,
            style::Print('\n'),
            style::Print(boxed_content),
            cursor::MoveUp(box_height - 1),
            cursor::MoveToColumn(2),
            style::Print(format!(" {} ", self.title)),
            cursor::MoveDown(box_height - 1)
        )?)
    }
}
