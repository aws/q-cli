use std::io::Write;

use crossterm::cursor::MoveTo;
use crossterm::{
    style,
    QueueableCommand,
};

use crate::Color;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Cell {
    pub symbol: char,
    pub foreground: Color,
    pub background: Color,
}

impl Default for Cell {
    fn default() -> Self {
        Self {
            symbol: ' ',
            foreground: Color::Reset,
            background: Color::Reset,
        }
    }
}

#[derive(Debug)]
pub struct DisplayState {
    width: u16,
    height: u16,
    old_cells: Vec<Vec<Cell>>,
    cells: Vec<Vec<Cell>>,
}

impl DisplayState {
    pub fn width(&self) -> u16 {
        self.width
    }

    pub fn height(&self) -> u16 {
        self.height
    }

    pub fn clear(&mut self) -> &mut Self {
        for row in &mut self.cells {
            for cell in row {
                *cell = Cell::default();
            }
        }
        self
    }

    pub fn draw_symbol(&mut self, symbol: char, x: u16, y: u16, foreground: Color, background: Color) -> &mut Self {
        if let Some(row) = self.cells.get_mut(usize::from(y)) {
            if let Some(cell) = row.get_mut(usize::from(x)) {
                *cell = Cell {
                    symbol,
                    foreground,
                    background,
                };
            }
        }
        self
    }

    pub fn draw_string(
        &mut self,
        string: impl AsRef<str>,
        x: u16,
        y: u16,
        foreground: Color,
        background: Color,
    ) -> &mut Self {
        if string.as_ref().is_empty() {
            return self;
        }

        for (i, symbol) in string.as_ref().chars().enumerate() {
            self.draw_symbol(symbol, x + u16::try_from(i).unwrap(), y, foreground, background);
        }

        self
    }

    #[allow(clippy::too_many_arguments)]
    pub fn draw_rect(
        &mut self,
        symbol: char,
        x: u16,
        y: u16,
        width: u16,
        height: u16,
        foreground: Color,
        background: Color,
    ) -> &mut Self {
        if width == 0 || height == 0 {
            return self;
        }

        for y in y..(y + height).min(self.height) {
            for x in x..(x + width).min(self.width) {
                self.cells[usize::from(y)][usize::from(x)] = Cell {
                    symbol,
                    foreground,
                    background,
                }
            }
        }

        self
    }

    pub(crate) fn new(size: (u16, u16)) -> Self {
        Self {
            width: size.0,
            height: size.1,
            old_cells: vec![vec![Cell::default(); size.0.into()]; size.1.into()],
            cells: vec![vec![Cell::default(); size.0.into()]; size.1.into()],
        }
    }

    pub(crate) fn resize(&mut self, new_width: u16, new_height: u16) {
        let new_width = usize::from(new_width);
        let new_height = usize::from(new_height);

        let diff_cell = Cell {
            symbol: '-',
            ..Default::default()
        };

        for y in 0..usize::from(self.height) {
            self.old_cells[y] = vec![diff_cell; new_width];
            self.cells[y] = vec![Cell::default(); new_width];
        }

        self.old_cells.resize(new_height, vec![diff_cell; new_width]);
        self.cells.resize(new_height, vec![Cell::default(); new_width]);

        self.width = u16::try_from(new_width).unwrap();
        self.height = u16::try_from(new_height).unwrap();
    }

    pub(crate) fn write_diff(&mut self, buf: &mut impl Write) -> std::io::Result<()> {
        for y in 0..self.height {
            let mut out_of_place = true;
            for x in 0..self.width {
                let (ix, iy) = (usize::from(x), usize::from(y));
                let cell = self.cells[iy][ix];
                match cell == self.old_cells[iy][ix] {
                    true => out_of_place = true,
                    false => {
                        if out_of_place {
                            out_of_place = false;
                            buf.queue(MoveTo(x, y))?;
                        }

                        buf.queue(style::SetForegroundColor(cell.foreground))?
                            .queue(style::SetBackgroundColor(cell.background))?
                            .queue(style::Print(cell.symbol))?;
                    },
                }
            }
        }

        std::mem::swap(&mut self.old_cells, &mut self.cells);
        buf.flush()
    }
}
