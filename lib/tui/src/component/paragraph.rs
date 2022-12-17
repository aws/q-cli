use termwiz::cell::{
    CellAttributes,
    Intensity,
};
use termwiz::color::ColorAttribute;
use termwiz::surface::Surface;
use unicode_segmentation::UnicodeSegmentation;
use unicode_width::UnicodeWidthStr;

use super::ComponentData;
use crate::surface_ext::SurfaceExt;
use crate::{
    Component,
    State,
};

#[derive(Debug)]
pub struct Paragraph {
    components: Vec<(String, Option<CellAttributes>)>,
    inner: ComponentData,
}

impl Paragraph {
    pub fn new(id: impl Into<String>) -> Self {
        Self {
            components: vec![],
            inner: ComponentData::new("p".to_owned(), id.into(), false),
        }
    }

    pub fn push_text(mut self, text: impl Into<String>) -> Self {
        self.components.push((text.into(), None));
        self
    }

    pub fn push_styled_text(
        mut self,
        text: impl Into<String>,
        foreground: ColorAttribute,
        background: ColorAttribute,
        bold: bool,
        italic: bool,
    ) -> Self {
        let mut attributes = CellAttributes::blank();
        attributes
            .set_foreground(foreground)
            .set_background(background)
            .set_intensity(if bold { Intensity::Bold } else { Intensity::Normal })
            .set_italic(italic);

        self.components.push((text.into(), Some(attributes)));
        self
    }
}

impl Component for Paragraph {
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

        let start_x = x;
        let start_y = y;
        for component in &self.components {
            let mut new_line = false;
            component.0.lines().for_each(|line| {
                if new_line {
                    x = start_x;
                    y += 1.0;
                }

                if y > start_y + height {
                    return;
                }

                surface.draw_text(
                    line,
                    x,
                    y,
                    width - (x - start_x),
                    component.1.clone().unwrap_or_else(|| style.attributes()),
                );
                x += line.width() as f64;

                new_line = true;
            });
        }
    }

    fn inner(&self) -> &super::ComponentData {
        &self.inner
    }

    fn inner_mut(&mut self) -> &mut super::ComponentData {
        &mut self.inner
    }

    fn size(&self, _: &mut State) -> (f64, f64) {
        let (width, height) = self.components.iter().fold((0, 1), |(acc0, acc1), c| {
            let width = c.0.lines().map(|t| t.width()).max().unwrap_or_default();
            let height = c.0.graphemes(true).filter(|s| s == &"\n" || s == &"\r\n").count();
            (acc0 + width, acc1 + height)
        });

        (width as f64, height as f64)
    }
}
