const MAX_CHARS_PER_CELL: usize = 6;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct TerminalPosition {
    row: usize,
    col: usize,
}

impl TerminalPosition {
    fn to_index(&self, row_width: usize) -> usize {
        self.row * row_width + self.col
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum TermColor {
    Rgb(u8, u8, u8),
    Idx(u8),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TermAttribs {
    in_prompt: bool,
    in_suggestion: bool,
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
    fn new(row: usize, col: usize) -> Cursor {
        Cursor(TerminalPosition { row, col })
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

    fn resize(&mut self, new_rows: usize, new_cols: usize) {}

    fn get(&self, position: TerminalPosition) -> Option<&ScreenCell> {
        if position.row >= self.width || position.col >= self.height {
            return None;
        }
        self.buffer.get(position.to_index(self.width))
    }

    fn get_mut(&mut self, position: TerminalPosition) -> Option<&mut ScreenCell> {
        if position.row >= self.width || position.col >= self.height {
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

    screen_attribs: TermAttribs,
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
    }

    fn resize(&mut self, new_rows: usize, new_cols: usize) {}
}
