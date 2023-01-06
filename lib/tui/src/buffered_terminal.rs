use std::ops::{
    Deref,
    DerefMut,
};

use termwiz::color::ColorAttribute;
use termwiz::surface::{
    Change,
    CursorVisibility,
    Position,
    Surface,
};
use termwiz::terminal::Terminal;

pub struct BufferedTerminal<T: Terminal> {
    terminal: T,
    surface: Surface,
    backbuffer: Surface,
}

impl<T: Terminal> BufferedTerminal<T> {
    pub fn new(mut terminal: T) -> termwiz::Result<Self> {
        let size = terminal.get_screen_size()?;
        let surface = Surface::new(size.cols, size.rows);
        let backbuffer = Surface::new(size.cols, size.rows);

        Ok(Self {
            terminal,
            surface,
            backbuffer,
        })
    }

    pub fn terminal(&mut self) -> &mut T {
        &mut self.terminal
    }

    pub fn flush(&mut self) -> termwiz::Result<()> {
        let mut changes = vec![Change::CursorVisibility(CursorVisibility::Hidden)];
        changes.append(&mut self.surface.diff_screens(&self.backbuffer));
        changes.append(&mut vec![
            Change::CursorPosition {
                x: Position::Absolute(self.backbuffer.cursor_position().0),
                y: Position::Absolute(self.backbuffer.cursor_position().1),
            },
            Change::CursorVisibility(self.backbuffer.cursor_visibility()),
        ]);

        self.terminal.render(&changes)?;

        let seqno = self.surface.add_changes(changes);
        self.surface.flush_changes_older_than(seqno);

        let seqno = self.backbuffer.current_seqno();
        self.backbuffer.flush_changes_older_than(seqno);

        Ok(())
    }

    pub fn resize(&mut self, width: usize, height: usize) -> termwiz::Result<()> {
        self.add_change(Change::ClearScreen(ColorAttribute::Default));
        self.flush()?;
        self.backbuffer.resize(width, height);
        self.surface.resize(width, height);
        Ok(())
    }
}

impl<T: Terminal> Deref for BufferedTerminal<T> {
    type Target = Surface;

    fn deref(&self) -> &Surface {
        &self.backbuffer
    }
}

impl<T: Terminal> DerefMut for BufferedTerminal<T> {
    fn deref_mut(&mut self) -> &mut Surface {
        &mut self.backbuffer
    }
}
