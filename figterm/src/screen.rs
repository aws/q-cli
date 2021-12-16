const MAX_CHARS_PER_CELL: usize = 6;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ScreenAttribs {
    in_prompt: bool,
    in_suggestion: bool,
}

impl ScreenAttribs {
    pub fn new() -> ScreenAttribs {
        ScreenAttribs {
            in_prompt: false,
            in_suggestion: false,
        }
    }
}

pub struct ScreenCell {
    chars: [char; MAX_CHARS_PER_CELL],
    attribs: ScreenAttribs,
}

impl ScreenCell {
    pub fn new() -> ScreenCell {
        ScreenCell {
            chars: ['\0'; MAX_CHARS_PER_CELL],
            attribs: ScreenAttribs::new(),
        }
    }

    pub fn set(&mut self, screen: &FigtermScreen, val: char) {
        self.chars[0] = val;
        self.attribs = screen.attribs;
    }

    pub fn clear(&mut self) {
        self.chars[0] = '\0';
    }
}

enum BufferTypes {
    Primary,
    Seconday,
}

pub struct FigtermScreen {
    rows: usize,
    cols: usize,

    current_buffer: BufferTypes,
    primary_buffer: Vec<ScreenCell>,
    seconday_buffer: Vec<ScreenCell>,

    attribs: ScreenAttribs,
}

impl FigtermScreen {
    // fn new() -> FigtermScreen {
    //     FigtermScreen {
    //         rows: (),
    //         cols: (),
    //         current_buffer: (),
    //         primary_buffer: (),
    //         seconday_buffer: (),
    //         attribs: (),
    //     }
    // }

    fn get_cell(&self, row: usize, col: usize) -> Option<&ScreenCell> {
        if row >= self.rows || col >= self.cols {
            return None;
        }

        self.get_current_buffer().get(self.cols * row + col)
    }

    fn get_current_buffer(&self) -> &Vec<ScreenCell> {
        match self.current_buffer {
            BufferTypes::Primary => &self.primary_buffer,
            BufferTypes::Seconday => &self.seconday_buffer,
        }
    }

    fn get_current_buffer_mut(&mut self) -> &mut Vec<ScreenCell> {
        match self.current_buffer {
            BufferTypes::Primary => &mut self.primary_buffer,
            BufferTypes::Seconday => &mut self.seconday_buffer,
        }
    }
}
