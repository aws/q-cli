use std::fmt;

use crate::ansi::{Column, Line};

const MAX_CHARS_PER_CELL: usize = 6;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct TerminalPosition {
    line: Line,
    column: Column,
}

impl TerminalPosition {
    fn to_index(&self, row_width: usize) -> usize {
        self.column * row_width + self.line as usize
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum TermColor {
    Rgb(u8, u8, u8),
    Idx(u8),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TermAttribs {
    pub in_prompt: bool,
    pub in_suggestion: bool,
    fg: TermColor,
    bg: TermColor,
}

impl TermAttribs {
    pub fn new() -> TermAttribs {
        TermAttribs {
            in_prompt: false,
            in_suggestion: false,
            fg: TermColor::Idx(0),
            bg: TermColor::Idx(0),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ScreenCell {
    chars: [char; MAX_CHARS_PER_CELL],
    attribs: TermAttribs,
}

impl ScreenCell {
    pub fn new() -> ScreenCell {
        ScreenCell {
            chars: ['\0'; MAX_CHARS_PER_CELL],
            attribs: TermAttribs::new(),
        }
    }

    pub fn set(&mut self, screen: &FigtermScreen, val: char) {
        self.chars[0] = val;
        self.attribs = screen.screen_attribs;
    }

    pub fn clear(&mut self) {
        self.chars[0] = '\0';
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Cursor(TerminalPosition);

impl Cursor {
    fn new(column: Column, line: Line) -> Cursor {
        Cursor(TerminalPosition { column, line })
    }
}

impl fmt::Display for Cursor {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}, {}", self.0.column, self.0.line)
    }
}

enum BufferType {
    Primary,
    Seconday,
}

pub struct ScreenBuffer {
    width: usize,
    height: usize,
    buffer: Vec<ScreenCell>,
}

impl ScreenBuffer {
    fn new(width: usize, height: usize) -> ScreenBuffer {
        ScreenBuffer {
            width,
            height,
            buffer: vec![ScreenCell::new(); width * height],
        }
    }

    fn resize(&mut self, _new_rows: usize, _new_cols: usize) {}

    fn get(&self, position: TerminalPosition) -> Option<&ScreenCell> {
        if position.column >= self.width || position.line >= self.height as i32 {
            return None;
        }
        self.buffer.get(position.to_index(self.width))
    }

    fn get_mut(&mut self, position: TerminalPosition) -> Option<&mut ScreenCell> {
        if position.column >= self.width || position.line >= self.height as i32 {
            return None;
        }
        self.buffer.get_mut(position.to_index(self.width))
    }
}

pub struct FigtermScreen {
    pub cursor: Cursor,

    current_buffer: BufferType,
    primary_buffer: ScreenBuffer,
    seconday_buffer: ScreenBuffer,

    pub screen_attribs: TermAttribs,
}

impl FigtermScreen {
    pub fn new() -> FigtermScreen {
        FigtermScreen {
            cursor: Cursor::new(0, 0),

            current_buffer: BufferType::Primary,
            primary_buffer: ScreenBuffer::new(0, 0),
            seconday_buffer: ScreenBuffer::new(0, 0),

            screen_attribs: TermAttribs::new(),
        }
    }

    fn get_cell(&self, position: TerminalPosition) -> Option<&ScreenCell> {
        self.get_current_buffer().get(position)
    }

    fn set_cell(&mut self, position: TerminalPosition, cell: ScreenCell) -> Option<()> {
        *self.get_current_buffer_mut().get_mut(position)? = cell;
        Some(())
    }

    fn get_current_buffer(&self) -> &ScreenBuffer {
        match self.current_buffer {
            BufferType::Primary => &self.primary_buffer,
            BufferType::Seconday => &self.seconday_buffer,
        }
    }

    fn get_current_buffer_mut(&mut self) -> &mut ScreenBuffer {
        match self.current_buffer {
            BufferType::Primary => &mut self.primary_buffer,
            BufferType::Seconday => &mut self.seconday_buffer,
        }
    }

    pub fn write(&mut self, c: char) {
        self.set_cell(
            self.cursor.0,
            ScreenCell {
                chars: [c, '\0', '\0', '\0', '\0', '\0'],
                attribs: TermAttribs {
                    in_prompt: self.screen_attribs.in_prompt,
                    in_suggestion: self.screen_attribs.in_suggestion,
                    fg: self.screen_attribs.fg,
                    bg: self.screen_attribs.bg,
                },
            },
        );

        // self.cursor.0.column += 1;
        // if self.cursor.0.column >= self.get_current_buffer().width {
        //     self.cursor.0.column = 0;
        //     self.cursor.0.line += 1;
        // }

        let mut screen = String::new();

        for y in 0..self.primary_buffer.height {
            for x in 0..self.primary_buffer.width {
                let cell = self.get_cell(TerminalPosition {
                    column: x,
                    line: y as i32,
                });
                if let Some(cell) = cell {
                    let mut chars = cell.chars;
                    if chars[0] == '\0' {
                        chars[0] = ' ';
                    }
                    let mut fg = cell.attribs.fg;
                    let mut bg = cell.attribs.bg;
                    if cell.attribs.in_prompt {
                        fg = TermColor::Idx(1);
                    }
                    if cell.attribs.in_suggestion {
                        bg = TermColor::Idx(1);
                    }
                    let fg_str = match fg {
                        TermColor::Idx(idx) => format!("\x1b[38;5;{}m", idx),
                        TermColor::Rgb(r, g, b) => format!("\x1b[38;2;{};{};{}m", r, g, b),
                    };
                    let bg_str = match bg {
                        TermColor::Idx(idx) => format!("\x1b[48;5;{}m", idx),
                        TermColor::Rgb(r, g, b) => format!("\x1b[48;2;{};{};{}m", r, g, b),
                    };
                    let reset_str = "\x1b[0m";
                    let mut line = String::new();
                    for c in chars.iter() {
                        line.push_str(&format!("{}{}{}", fg_str, c, reset_str));
                    }
                    line.push_str(&format!("{}{}", bg_str, reset_str));
                    screen.push_str(&line);
                }
            }
            screen.push_str("\n");
        }

        log::info!("{}", screen);
    }

    pub fn resize(&mut self, _new_rows: usize, _new_cols: usize) {}

    pub fn goto(&mut self, line: Line, column: Column) {
        self.cursor.0.line = line;
        self.cursor.0.column = column;
    }

    pub fn goto_line(&mut self, line: Line) {
        self.cursor.0.line = line;
    }

    pub fn goto_column(&mut self, column: Column) {
        self.cursor.0.column = column;
    }

    pub fn move_up(&mut self, n: usize) {
        self.cursor.0.line -= n as i32;
    }

    pub fn move_down(&mut self, n: usize) {
        self.cursor.0.line += n as i32;
    }

    pub fn move_forward(&mut self, columns: Column) {
        self.cursor.0.column += columns;
    }

    pub fn move_backward(&mut self, columns: Column) {
        let _ = self.cursor.0.column.saturating_sub(columns);
    }
}
