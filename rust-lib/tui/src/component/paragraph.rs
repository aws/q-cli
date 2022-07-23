use newton::{
    Color,
    DisplayState,
};

use crate::Style;

#[derive(Debug)]
pub enum ParagraphComponent {
    Text {
        text: String,
        color: Option<Color>,
        background_color: Option<Color>,
    },
    LineBreak,
}

#[derive(Debug, Default)]
pub struct Paragraph {
    pub components: Vec<ParagraphComponent>,
}

impl Paragraph {
    pub fn new() -> Self {
        Default::default()
    }

    pub fn push_text(&mut self, text: impl Into<String>) {
        self.push_styled_text(text, None, None);
    }

    pub fn push_styled_text(&mut self, text: impl Into<String>, color: Option<Color>, background_color: Option<Color>) {
        self.components.push(ParagraphComponent::Text {
            text: text.into(),
            color,
            background_color,
        })
    }

    pub fn push_line_break(&mut self) {
        self.components.push(ParagraphComponent::LineBreak);
    }

    pub(crate) fn initialize(&self, width: &mut i32, height: &mut i32) {
        *width = 110;
        if !self.components.is_empty() {
            *height = self.components.iter().fold(1, |acc, c| match c {
                ParagraphComponent::Text { text, .. } => {
                    acc + i32::try_from(text.chars().filter(|c| c == &'\n').count()).unwrap()
                },
                ParagraphComponent::LineBreak => acc + 1,
            }) + 5;
        }
    }

    pub(crate) fn draw(
        &self,
        renderer: &mut DisplayState,
        style: &Style,
        mut x: i32,
        mut y: i32,
        width: i32,
        height: i32,
    ) {
        let start = x;
        let mut offset = 0;
        for component in &self.components {
            if y == height {
                return;
            }

            match component {
                ParagraphComponent::Text {
                    text,
                    color,
                    background_color,
                } => {
                    for char in text.chars() {
                        if char == '\n' {
                            x = start;
                            y += 1;
                            offset = 0;
                            continue;
                        }

                        renderer.draw_symbol(
                            char,
                            x + offset,
                            y,
                            color.unwrap_or(style.color()),
                            background_color.unwrap_or(style.background_color()),
                        );
                        x += 1;

                        if x >= width {
                            x = start;
                            y += 1;
                            offset = 4;
                        }
                    }
                },
                ParagraphComponent::LineBreak => {
                    x = start;
                    y += 1;
                    offset = 0;
                },
            }
        }
    }
}
