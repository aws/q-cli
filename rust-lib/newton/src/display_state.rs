use std::io::Write;

use crossterm::style::{
    Attribute,
    Colors,
};
use crossterm::terminal::{
    self,
    ClearType,
};
use crossterm::{
    cursor,
    style,
    QueueableCommand,
};

use crate::Color;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Cell {
    pub symbol: char,
    pub color: Color,
    pub background_color: Color,
    pub bold: bool,
}

impl Default for Cell {
    fn default() -> Self {
        Self {
            symbol: ' ',
            color: Color::Reset,
            background_color: Color::Reset,
            bold: false,
        }
    }
}

#[derive(Debug)]
pub struct DisplayState {
    width: u16,
    height: u16,
    pub(crate) starting_row: u16,
    cells: Vec<Vec<Cell>>,
    lines: Vec<Vec<u8>>,
}

impl DisplayState {
    pub fn width(&self) -> i32 {
        self.width.into()
    }

    pub fn height(&self) -> i32 {
        self.height.into()
    }

    pub fn clear(&mut self) -> &mut Self {
        for row in &mut self.cells {
            for cell in row {
                *cell = Cell::default();
            }
        }
        self
    }

    pub fn draw_symbol(
        &mut self,
        symbol: char,
        x: i32,
        y: i32,
        color: Color,
        background_color: Color,
        bold: bool,
    ) -> &mut Self {
        let (x, y) = match (usize::try_from(x), usize::try_from(y)) {
            (Ok(x), Ok(y)) => (x, y),
            _ => return self,
        };

        if let Some(row) = self.cells.get_mut(y) {
            if let Some(cell) = row.get_mut(x) {
                *cell = Cell {
                    symbol,
                    color,
                    background_color,
                    bold,
                };
            }
        }
        self
    }

    pub fn draw_string(
        &mut self,
        string: impl AsRef<str>,
        x: i32,
        y: i32,
        color: Color,
        background_color: Color,
        bold: bool,
    ) -> &mut Self {
        for (i, symbol) in string.as_ref().chars().enumerate() {
            self.draw_symbol(symbol, x + i32::try_from(i).unwrap(), y, color, background_color, bold);
        }

        self
    }

    #[allow(clippy::too_many_arguments)]
    pub fn draw_rect(
        &mut self,
        symbol: char,
        x: i32,
        y: i32,
        width: i32,
        height: i32,
        color: Color,
        background_color: Color,
    ) -> &mut Self {
        for y in y..(y + height).min(self.height()) {
            for x in x..(x + width).min(self.width()) {
                self.draw_symbol(symbol, x, y, color, background_color, false);
            }
        }

        self
    }

    pub(crate) fn new(size: (u16, u16), starting_row: u16) -> Self {
        Self {
            width: size.0,
            height: size.1,
            starting_row,
            cells: vec![vec![Cell::default(); size.0.into()]; size.1.into()],
            lines: vec![vec![]; size.1.into()],
        }
    }

    pub(crate) fn resize(&mut self, buf: &mut impl Write, width: i32, height: i32) -> std::io::Result<()> {
        for line in &mut self.lines {
            line.clear();
        }

        let (width, height) = match (u16::try_from(width), u16::try_from(height)) {
            (Ok(x), Ok(y)) => (x, y),
            _ => return Ok(()),
        };

        self.width = width;
        self.height = height;

        for row in &mut self.cells {
            row.resize(self.width.into(), Cell::default())
        }
        self.cells
            .resize(self.height.into(), vec![Cell::default(); self.width.into()]);
        self.lines.resize(self.height.into(), vec![]);

        Ok(())
    }

    pub(crate) fn write_diff(&mut self, buf: &mut impl Write) -> std::io::Result<()> {
        buf.queue(cursor::MoveTo(0, self.starting_row))?;
        for y in 0..self.height {
            let mut line = vec![];
            let mut color = Color::Reset;
            let mut background_color = Color::Reset;
            let mut bold = false;

            for cell in &self.cells[usize::from(y)] {
                if cell.color != color {
                    color = cell.color;
                    line.queue(style::SetForegroundColor(color))?;
                }

                if cell.background_color != background_color {
                    background_color = cell.background_color;
                    line.queue(style::SetBackgroundColor(background_color))?;
                }

                if cell.bold != bold {
                    bold = cell.bold;
                    line.queue(style::SetAttribute(Attribute::Bold))?;
                }

                line.queue(style::Print(cell.symbol))?;
            }

            line.queue(style::SetColors(Colors::new(Color::Reset, Color::Reset)))?;
            line.queue(style::SetAttribute(Attribute::NormalIntensity))?;

            if self.lines[usize::from(y)].len() != line.len()
                || self.lines[usize::from(y)]
                    .iter()
                    .enumerate()
                    .any(|(i, byte)| *byte != line[i])
            {
                buf.queue(terminal::Clear(ClearType::CurrentLine))?.write_all(&line)?;
                self.lines[usize::from(y)] = line;
            }

            buf.queue(cursor::MoveToNextLine(1))?;
        }

        buf.flush()
    }
}
