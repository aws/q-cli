use termwiz::color::ColorAttribute;
use termwiz::surface::Surface;

use super::ComponentData;
use crate::surface_ext::SurfaceExt;
use crate::Component;

#[derive(Debug)]
enum ParagraphComponent {
    Text {
        text: String,
        color: Option<ColorAttribute>,
        background_color: Option<ColorAttribute>,
        bold: bool,
    },
    LineBreak,
}

#[derive(Debug)]
pub struct Paragraph {
    components: Vec<ParagraphComponent>,
    inner: ComponentData,
}

impl Paragraph {
    pub fn new(id: impl Into<String>) -> Self {
        Self {
            components: vec![],
            inner: ComponentData::new(id.into(), false),
        }
    }

    pub fn push_text(self, text: impl Into<String>) -> Self {
        self.push_styled_text(text, None, None, false)
    }

    pub fn push_styled_text(
        mut self,
        text: impl Into<String>,
        color: Option<ColorAttribute>,
        background_color: Option<ColorAttribute>,
        bold: bool,
    ) -> Self {
        self.components.push(ParagraphComponent::Text {
            text: text.into().replace('\t', "    "),
            color,
            background_color,
            bold,
        });

        self
    }

    pub fn push_line_break(mut self) -> Self {
        self.components.push(ParagraphComponent::LineBreak);
        self
    }
}

impl Component for Paragraph {
    fn initialize(&mut self, _: &mut crate::event_loop::State) {
        self.inner.width = 110.0;
        if !self.components.is_empty() {
            self.inner.height = self.components.iter().fold(1, |acc, c| match c {
                ParagraphComponent::Text { text, .. } => {
                    acc + i32::try_from(text.chars().filter(|c| c == &'\n').count()).unwrap()
                },
                ParagraphComponent::LineBreak => acc + 1,
            }) as f64;
        }
    }

    fn draw(
        &self,
        state: &mut crate::event_loop::State,
        surface: &mut Surface,
        mut x: f64,
        mut y: f64,
        width: f64,
        height: f64,
        _: f64,
        _: f64,
    ) {
        let style = self.style(state);

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
                    bold,
                } => {
                    for char in text.chars() {
                        if char == '\n' {
                            x = start;
                            y += 1.0;
                            offset = 0;
                            continue;
                        }

                        surface.draw_text(
                            char,
                            x + offset as f64,
                            y,
                            color.unwrap_or(style.color()),
                            background_color.unwrap_or(style.background_color()),
                            *bold,
                        );
                        x += 1.0;

                        if x >= width {
                            x = start;
                            y += 1.0;
                            offset = 4;
                        }
                    }
                },
                ParagraphComponent::LineBreak => {
                    x = start;
                    y += 1.0;
                    offset = 0;
                },
            }
        }
    }

    fn class(&self) -> &'static str {
        "p"
    }

    fn inner(&self) -> &super::ComponentData {
        &self.inner
    }

    fn inner_mut(&mut self) -> &mut super::ComponentData {
        &mut self.inner
    }
}
