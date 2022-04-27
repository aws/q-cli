//! Exports the `Term` type which is a high-level API for the Grid.

use std::cmp::{max, min};
use std::ops::{Index, IndexMut, Range};
use std::path::PathBuf;
use std::{env, mem, ptr, str};

use bitflags::bitflags;
use log::{debug, trace};
use serde::{Deserialize, Serialize};
use unicode_width::UnicodeWidthChar;

use crate::ansi::{self, Attr, CharsetIndex, Color, Handler, NamedColor, StandardCharset};
use crate::event::{Event, EventListener};
use crate::grid::{Dimensions, Grid, GridIterator, Scroll};
use crate::index::{self, Boundary, Column, Direction, Line, Point, Rect};
use crate::term::cell::{Cell, LineLength, ShellFlags};
use crate::term::color::{Colors, Rgb};

use self::cell::FigFlags;

pub mod cell;
pub mod color;

/// Minimum number of columns.
///
/// A minimum of 2 is necessary to hold fullwidth unicode characters.
pub const MIN_COLUMNS: usize = 2;

/// Minimum number of visible lines.
pub const MIN_SCREEN_LINES: usize = 1;

/// Max size of the window title stack.
const TITLE_STACK_MAX_DEPTH: usize = 4096;

/// Default tab interval, corresponding to terminfo `it` value.
const INITIAL_TABSTOPS: usize = 8;

bitflags! {
    pub struct TermMode: u32 {
        const NONE                = 0;
        const SHOW_CURSOR         = 0b0000_0000_0000_0000_0001;
        const APP_CURSOR          = 0b0000_0000_0000_0000_0010;
        const APP_KEYPAD          = 0b0000_0000_0000_0000_0100;
        const MOUSE_REPORT_CLICK  = 0b0000_0000_0000_0000_1000;
        const BRACKETED_PASTE     = 0b0000_0000_0000_0001_0000;
        const SGR_MOUSE           = 0b0000_0000_0000_0010_0000;
        const MOUSE_MOTION        = 0b0000_0000_0000_0100_0000;
        const LINE_WRAP           = 0b0000_0000_0000_1000_0000;
        const LINE_FEED_NEW_LINE  = 0b0000_0000_0001_0000_0000;
        const ORIGIN              = 0b0000_0000_0010_0000_0000;
        const INSERT              = 0b0000_0000_0100_0000_0000;
        const FOCUS_IN_OUT        = 0b0000_0000_1000_0000_0000;
        const ALT_SCREEN          = 0b0000_0001_0000_0000_0000;
        const MOUSE_DRAG          = 0b0000_0010_0000_0000_0000;
        const MOUSE_MODE          = 0b0000_0010_0000_0100_1000;
        const UTF8_MOUSE          = 0b0000_0100_0000_0000_0000;
        const ALTERNATE_SCROLL    = 0b0000_1000_0000_0000_0000;
        const VI                  = 0b0001_0000_0000_0000_0000;
        const URGENCY_HINTS       = 0b0010_0000_0000_0000_0000;
        const ANY                 = std::u32::MAX;
    }
}

impl Default for TermMode {
    fn default() -> TermMode {
        TermMode::SHOW_CURSOR
            | TermMode::LINE_WRAP
            | TermMode::ALTERNATE_SCROLL
            | TermMode::URGENCY_HINTS
    }
}

/// Terminal size info.
#[derive(Serialize, Deserialize, Debug, Copy, Clone, PartialEq)]
pub struct SizeInfo {
    /// Number of lines in the viewport.
    screen_lines: usize,

    /// Number of columns in the viewport.
    columns: usize,
}

impl SizeInfo {
    pub fn new(screen_lines: usize, columns: usize) -> SizeInfo {
        SizeInfo {
            screen_lines,
            columns,
        }
    }

    #[inline]
    pub fn reserve_lines(&mut self, count: usize) {
        self.screen_lines = max(self.screen_lines.saturating_sub(count), MIN_SCREEN_LINES);
    }
}

impl Dimensions for SizeInfo {
    #[inline]
    fn columns(&self) -> usize {
        self.columns
    }

    #[inline]
    fn screen_lines(&self) -> usize {
        self.screen_lines
    }

    #[inline]
    fn total_lines(&self) -> usize {
        self.screen_lines()
    }
}

/// Information about the current command
#[derive(Debug, Clone)]
pub struct CommandInfo {
    pub command: Option<String>,
    pub shell: Option<String>,
    pub pid: Option<i32>,
    pub session_id: Option<String>,
    pub cwd: Option<PathBuf>,
    pub time: Option<u64>,

    pub hostname: Option<String>,
    pub in_ssh: bool,
    pub in_docker: bool,

    pub exit_code: Option<i32>,
}

#[derive(Debug, Clone, Default)]
pub struct ShellContext {
    /// Pid of the shell
    pub pid: Option<i32>,
    /// The current tty
    pub tty: Option<String>,
    /// The name of the current shell's executable
    pub shell: Option<String>,
    /// Current working directory
    pub current_working_directory: Option<PathBuf>,
    /// The current session id
    pub session_id: Option<String>,
    /// Hostname of the machine where the shell is running
    pub hostname: Option<String>,
}

/// State about the current shell
#[derive(Debug, Clone, Default)]
pub struct ShellState {
    /// Local context for shell
    pub local_context: ShellContext,
    /// Remote context for shell
    pub remote_context: ShellContext,
    /// If the shell is running in a ssh session
    pub in_ssh: bool,
    /// If the shell is running in a docker session
    pub in_docker: bool,
    /// If the prompt has been seen
    pub has_seen_prompt: bool,
    /// PreExec
    pub preexec: bool,
    /// Position of start of cmd
    pub cmd_cursor: Option<Point>,
    /// Fish suggestion color
    pub fish_suggestion_color: Option<fig_color::SuggestionColor>,
    /// Zsh autosuggestion color
    pub zsh_autosuggestion_color: Option<fig_color::SuggestionColor>,
    /// Color support
    pub color_support: Option<fig_color::ColorSupport>,
    /// Command info
    pub command_info: Option<CommandInfo>,
    /// Fig Log Level
    pub fig_log_level: Option<String>,
}

impl ShellState {
    fn new() -> ShellState {
        ShellState::default()
    }
}

impl ShellState {
    /// Get the current [`ShellContext`]
    pub fn get_context(&self) -> &ShellContext {
        if self.in_ssh || self.in_docker {
            &self.remote_context
        } else {
            &self.local_context
        }
    }

    /// Get the current [`ShellContext`]
    pub fn get_mut_context(&mut self) -> &mut ShellContext {
        if self.in_ssh || self.in_docker {
            &mut self.remote_context
        } else {
            &mut self.local_context
        }
    }
}

#[derive(Debug, Clone)]
pub struct TextBuffer {
    pub buffer: String,
    pub cursor_idx: Option<usize>,
}

pub struct Term<T> {
    /// Currently active grid.
    ///
    /// Tracks the screen buffer currently in use. While the alternate screen buffer is active,
    /// this will be the alternate grid. Otherwise it is the primary screen buffer.
    grid: Grid<Cell>,

    /// Currently inactive grid.
    ///
    /// Opposite of the active grid. While the alternate screen buffer is active, this will be the
    /// primary grid. Otherwise it is the alternate screen buffer.
    inactive_grid: Grid<Cell>,

    /// Index into `charsets`, pointing to what ASCII is currently being mapped to.
    active_charset: CharsetIndex,

    /// Tabstops.
    tabs: TabStops,

    /// Mode flags.
    mode: TermMode,

    /// Scroll region.
    ///
    /// Range going from top to bottom of the terminal, indexed from the top of the viewport.
    scroll_region: Range<Line>,

    /// Modified terminal colors.
    colors: Colors,

    /// Proxy for sending events to the event loop.
    event_proxy: T,

    /// Current title of the window.
    title: Option<String>,

    /// Stack of saved window titles. When a title is popped from this stack, the `title` for the
    /// term is set.
    title_stack: Vec<Option<String>>,

    /// State tracked by figterm to determine the current state of the shell
    shell_state: ShellState,
}

impl<T> Term<T> {
    #[inline]
    pub fn scroll_display(&mut self, scroll: Scroll)
    where
        T: EventListener,
    {
        self.grid.scroll_display(scroll);
    }

    pub fn new(size: SizeInfo, event_proxy: T, max_scroll_limit: usize) -> Term<T> {
        let num_cols = size.columns;
        let num_lines = size.screen_lines;

        // TODO: determine max_scroll_limit
        let grid = Grid::new(num_lines, num_cols, max_scroll_limit);
        let alt = Grid::new(num_lines, num_cols, 0);

        let tabs = TabStops::new(grid.columns());

        let scroll_region = Line(0)..Line(grid.screen_lines() as i32);

        let mut shell_state = ShellState::new();
        shell_state.get_mut_context().session_id = std::env::var("TERM_SESSION_ID").ok();
        shell_state.color_support = Some(fig_color::get_color_support());

        Term {
            grid,
            inactive_grid: alt,
            active_charset: Default::default(),
            tabs,
            mode: Default::default(),
            scroll_region,
            colors: color::Colors::default(),
            event_proxy,
            title: None,
            title_stack: Vec::new(),
            shell_state,
        }
    }

    /// Convert range between two points to a String.
    pub fn bounds_to_string(&self, start: Point, end: Point) -> String {
        let mut res = String::new();

        for line in (start.line.0..=end.line.0).map(Line::from) {
            let start_col = if line == start.line {
                start.column
            } else {
                Column(0)
            };
            let end_col = if line == end.line {
                end.column
            } else {
                self.last_column()
            };

            res += &self.line_to_string(line, start_col..end_col, line == end.line);
        }

        res
    }

    /// Convert a single line in the grid to a String.
    fn line_to_string(
        &self,
        line: Line,
        mut cols: Range<Column>,
        include_wrapped_wide: bool,
    ) -> String {
        let mut text = String::new();

        let grid_line = &self.grid[line];
        let line_length = min(grid_line.line_length(), cols.end + 1);

        // Include wide char when trailing spacer is selected.
        if grid_line[cols.start]
            .shell_flags
            .contains(ShellFlags::WIDE_CHAR_SPACER)
        {
            cols.start -= 1;
        }

        let mut tab_mode = false;
        for column in (cols.start.0..line_length.0).map(Column::from) {
            let cell = &grid_line[column];

            // Skip over cells until next tab-stop once a tab was found.
            if tab_mode {
                if self.tabs[column] || cell.c != ' ' {
                    tab_mode = false;
                } else {
                    continue;
                }
            }

            if cell.c == '\t' {
                tab_mode = true;
            }

            if !cell
                .shell_flags
                .intersects(ShellFlags::WIDE_CHAR_SPACER | ShellFlags::LEADING_WIDE_CHAR_SPACER)
            {
                // Push cells primary character.
                text.push(cell.c);

                // Push zero-width characters.
                for c in cell.zerowidth().into_iter().flatten() {
                    text.push(*c);
                }
            }
        }

        if cols.end >= self.columns() - 1
            && (line_length.0 == 0
                || !self.grid[line][line_length - 1]
                    .shell_flags
                    .contains(ShellFlags::WRAPLINE))
        {
            text.push('\n');
        }

        // If wide char is not part of the selection, but leading spacer is, include it.
        if line_length == self.columns()
            && line_length.0 >= 2
            && grid_line[line_length - 1]
                .shell_flags
                .contains(ShellFlags::LEADING_WIDE_CHAR_SPACER)
            && include_wrapped_wide
        {
            text.push(self.grid[line - 1i32][Column(0)].c);
        }

        text
    }

    /// Terminal content required for rendering.
    #[inline]
    pub fn renderable_content(&self) -> RenderableContent<'_>
    where
        T: EventListener,
    {
        RenderableContent::new(self)
    }

    /// Access to the raw grid data structure.
    ///
    /// This is a bit of a hack; when the window is closed, the event processor
    /// serializes the grid state to a file.
    pub fn grid(&self) -> &Grid<Cell> {
        &self.grid
    }

    /// Mutable access for swapping out the grid during tests.
    #[cfg(test)]
    pub fn grid_mut(&mut self) -> &mut Grid<Cell> {
        &mut self.grid
    }

    /// Resize terminal to new dimensions.
    pub fn resize(&mut self, size: SizeInfo) {
        let old_cols = self.columns();
        let old_lines = self.screen_lines();

        let num_cols = size.columns;
        let num_lines = size.screen_lines;

        if old_cols == num_cols && old_lines == num_lines {
            debug!("Term::resize dimensions unchanged");
            return;
        }

        debug!(
            "New num_cols is {} and num_lines is {}",
            num_cols, num_lines
        );

        // Invalidate selection and tabs only when necessary.
        if old_cols != num_cols {
            // Recreate tabs list.
            self.tabs.resize(num_cols);
        }

        let is_alt = self.mode.contains(TermMode::ALT_SCREEN);
        self.grid.resize(!is_alt, num_lines, num_cols);
        self.inactive_grid.resize(is_alt, num_lines, num_cols);

        // Reset scrolling region.
        self.scroll_region = Line(0)..Line(self.screen_lines() as i32);
    }

    /// Active terminal modes.
    #[inline]
    pub fn mode(&self) -> &TermMode {
        &self.mode
    }

    /// Swap primary and alternate screen buffer.
    pub fn swap_alt(&mut self) {
        if !self.mode.contains(TermMode::ALT_SCREEN) {
            // Set alt screen cursor to the current primary screen cursor.
            self.inactive_grid.cursor = self.grid.cursor.clone();

            // Drop information about the primary screens saved cursor.
            self.grid.saved_cursor = self.grid.cursor.clone();

            // Reset alternate screen contents.
            self.inactive_grid.reset_region(..);
        }

        mem::swap(&mut self.grid, &mut self.inactive_grid);
        self.mode ^= TermMode::ALT_SCREEN;
    }

    /// Scroll screen down.
    ///
    /// Text moves down; clear at bottom
    /// Expects origin to be in scroll range.
    #[inline]
    fn scroll_down_relative(&mut self, origin: Line, mut lines: usize) {
        trace!(
            "Scrolling down relative: origin={}, lines={}",
            origin,
            lines
        );

        if let Some(ref mut cursor) = self.shell_state.cmd_cursor {
            cursor.line += lines as i32;
        }

        lines = min(
            lines,
            (self.scroll_region.end - self.scroll_region.start).0 as usize,
        );
        lines = min(lines, (self.scroll_region.end - origin).0 as usize);

        let region = origin..self.scroll_region.end;

        // Scroll between origin and bottom
        self.grid.scroll_down(&region, lines);
    }

    /// Scroll screen up
    ///
    /// Text moves up; clear at top
    /// Expects origin to be in scroll range.
    #[inline]
    fn scroll_up_relative(&mut self, origin: Line, mut lines: usize) {
        trace!("Scrolling up relative: origin={}, lines={}", origin, lines);

        if let Some(ref mut cursor) = self.shell_state.cmd_cursor {
            cursor.line -= lines as i32;
        }

        lines = min(
            lines,
            (self.scroll_region.end - self.scroll_region.start).0 as usize,
        );

        let region = origin..self.scroll_region.end;

        self.grid.scroll_up(&region, lines);
    }

    fn deccolm(&mut self)
    where
        T: EventListener,
    {
        // Setting 132 column font makes no sense, but run the other side effects.
        // Clear scrolling region.
        self.set_scrolling_region(1, None);

        // Clear grid.
        self.grid.reset_region(..);
    }

    #[inline]
    pub fn exit(&mut self)
    where
        T: EventListener,
    {
        trace!("Exit");
    }

    pub fn scroll_to_point(&mut self, point: Point)
    where
        T: EventListener,
    {
        let display_offset = self.grid.display_offset() as i32;
        let screen_lines = self.grid.screen_lines() as i32;

        if point.line < -display_offset {
            let lines = point.line + display_offset;
            self.scroll_display(Scroll::Delta(-lines.0));
        } else if point.line >= (screen_lines - display_offset) {
            let lines = point.line + display_offset - screen_lines + 1i32;
            self.scroll_display(Scroll::Delta(-lines.0));
        }
    }

    /// Jump to the end of a wide cell.
    pub fn expand_wide(&self, mut point: Point, direction: Direction) -> Point {
        let flags = self.grid[point.line][point.column].shell_flags;

        match direction {
            Direction::Right if flags.contains(ShellFlags::LEADING_WIDE_CHAR_SPACER) => {
                point.column = Column(1);
                point.line += 1;
            }
            Direction::Right if flags.contains(ShellFlags::WIDE_CHAR) => {
                point.column = min(point.column + 1, self.last_column());
            }
            Direction::Left
                if flags.intersects(ShellFlags::WIDE_CHAR | ShellFlags::WIDE_CHAR_SPACER) =>
            {
                if flags.contains(ShellFlags::WIDE_CHAR_SPACER) {
                    point.column -= 1;
                }

                let prev = point.sub(self, Boundary::Grid, 1);
                if self.grid[prev]
                    .shell_flags
                    .contains(ShellFlags::LEADING_WIDE_CHAR_SPACER)
                {
                    point = prev;
                }
            }
            _ => (),
        }

        point
    }

    /// Insert a linebreak at the current cursor position.
    #[inline]
    fn wrapline(&mut self)
    where
        T: EventListener,
    {
        if !self.mode.contains(TermMode::LINE_WRAP) {
            return;
        }

        trace!("Wrapping input");

        self.grid
            .cursor_cell()
            .shell_flags
            .insert(ShellFlags::WRAPLINE);

        if self.grid.cursor.point.line + 1 >= self.scroll_region.end {
            self.linefeed();
        } else {
            self.grid.cursor.point.line += 1;
        }

        self.grid.cursor.point.column = Column(0);
        self.grid.cursor.input_needs_wrap = false;
    }

    /// Write `c` to the cell at the cursor position.
    #[inline(always)]
    fn write_at_cursor(&mut self, c: char) {
        let c = self.grid.cursor.charsets[self.active_charset].map(c);
        let fg = self.grid.cursor.template.fg;
        let bg = self.grid.cursor.template.bg;
        let shell_flags = self.grid.cursor.template.shell_flags;
        let fig_flags = self.grid.cursor.template.fig_flags;

        let mut cursor_cell = self.grid.cursor_cell();

        // Clear all related cells when overwriting a fullwidth cell.
        if cursor_cell
            .shell_flags
            .intersects(ShellFlags::WIDE_CHAR | ShellFlags::WIDE_CHAR_SPACER)
        {
            // Remove wide char and spacer.
            let wide = cursor_cell.shell_flags.contains(ShellFlags::WIDE_CHAR);
            let point = self.grid.cursor.point;
            if wide && point.column < self.last_column() {
                self.grid[point.line][point.column + 1]
                    .shell_flags
                    .remove(ShellFlags::WIDE_CHAR_SPACER);
            } else if point.column > 0 {
                self.grid[point.line][point.column - 1].clear_wide();
            }

            // Remove leading spacers.
            if point.column <= 1 && point.line != self.topmost_line() {
                let column = self.last_column();
                self.grid[point.line - 1i32][column]
                    .shell_flags
                    .remove(ShellFlags::LEADING_WIDE_CHAR_SPACER);
            }

            cursor_cell = self.grid.cursor_cell();
        }

        cursor_cell.drop_extra();

        cursor_cell.c = c;
        cursor_cell.fg = fg;
        cursor_cell.bg = bg;
        cursor_cell.shell_flags = shell_flags;
        cursor_cell.fig_flags = fig_flags;
    }

    /// Get the current [`ShellState`]
    pub fn shell_state(&self) -> &ShellState {
        &self.shell_state
    }

    pub fn get_text_region(
        &self,
        rect: &Rect,
        start_col_offset: Column,
        mask: Option<char>,
        wrap_lines: bool,
    ) -> Option<TextBuffer>
    where
        T: EventListener,
    {
        let mut buffer = String::with_capacity(rect.size());
        let mut padding: usize = 0;
        let cursor = self.grid().cursor.point;

        let mut last_char_width: usize = 0;
        let mut last_char_was_padding = true;

        let mut cell_idx = 0;
        let mut cursor_idx = None;

        let mut start = rect.start;
        start.column += start_col_offset;

        let end = rect.end;

        if start > end {
            return None;
        }

        for cell in self.grid().iter_from_to(start, end) {
            if cell.point.column == rect.start.column {
                last_char_was_padding = true;
            }

            if cell.point == cursor {
                cursor_idx = Some(cell_idx);
                while padding > 0 {
                    buffer.push(' ');
                    cell_idx += 1;
                    padding = padding.saturating_sub(1);
                }
            }

            if last_char_width > 0 {
                last_char_width = last_char_width.saturating_sub(1);
                continue;
            }

            if cell.c == '\0'
                || (mask == Some(' ')
                    && (cell.fig_flags.contains(FigFlags::IN_PROMPT)
                        || cell.fig_flags.contains(FigFlags::IN_SUGGESTION)))
            {
                padding = padding.saturating_add(1);
                last_char_was_padding = true;
            } else if cell.c as u32 == u32::MAX {
            } else {
                while padding > 0 {
                    buffer.push(' ');
                    cell_idx += 1;
                    padding = padding.saturating_sub(1);
                }

                match mask {
                    Some(mask)
                        if cell.fig_flags.contains(FigFlags::IN_PROMPT)
                            || cell.fig_flags.contains(FigFlags::IN_SUGGESTION) =>
                    {
                        buffer.push(mask);
                        cell_idx += 1;
                    }
                    _ => {
                        match cell.zerowidth() {
                            Some(zero_width) => {
                                buffer.push(cell.c);
                                for c in zero_width {
                                    buffer.push(*c);
                                }
                            }
                            None => {
                                buffer.push(cell.c);
                            }
                        }

                        last_char_width = cell.c.width().unwrap_or(1);
                        last_char_width = last_char_width.saturating_sub(1);

                        cell_idx += 1;
                        last_char_was_padding = false;
                    }
                }
            }

            if cell.point.column == rect.end.column.saturating_sub(1)
                && cell.point.line < rect.end.line
            {
                if last_char_was_padding || !wrap_lines {
                    buffer.push('\n');
                    cell_idx += 1;
                }
                padding = 0;
            }
        }

        Some(TextBuffer { buffer, cursor_idx })
    }

    pub fn get_current_buffer(&self) -> Option<TextBuffer>
    where
        T: EventListener,
    {
        match self.shell_state().cmd_cursor {
            Some(cmd_cursor) => {
                let start = Point::new(cmd_cursor.line, Column(0));
                let end = Point::new(self.bottommost_line(), self.last_column());

                if start < end {
                    let rect = Rect { start, end };

                    let mut buffer = self.get_text_region(
                        &rect,
                        Column(cmd_cursor.column.saturating_sub(1)),
                        Some(' '),
                        true,
                    )?;

                    if let Some(cursor_idx) = buffer.cursor_idx {
                        buffer.buffer = buffer.buffer.trim_end().to_string();

                        if buffer.buffer.len() < cursor_idx {
                            buffer.buffer.push_str(
                                &" ".repeat(cursor_idx.saturating_sub(buffer.buffer.len())),
                            );
                        }
                    }

                    Some(buffer)
                } else {
                    None
                }
            }
            None => None,
        }
    }
}

impl<T> Dimensions for Term<T> {
    #[inline]
    fn columns(&self) -> usize {
        self.grid.columns()
    }

    #[inline]
    fn screen_lines(&self) -> usize {
        self.grid.screen_lines()
    }

    #[inline]
    fn total_lines(&self) -> usize {
        self.grid.total_lines()
    }
}

impl<T: EventListener> Handler for Term<T> {
    /// A character to be displayed.
    #[inline(never)]
    fn input(&mut self, c: char) {
        trace!("Input: {}", c);

        // Number of cells the char will occupy.
        let width = match c.width() {
            Some(width) => width,
            None => return,
        };

        // Handle zero-width characters.
        if width == 0 {
            // Get previous column.
            let mut column = self.grid.cursor.point.column;
            if !self.grid.cursor.input_needs_wrap {
                column.0 = column.saturating_sub(1);
            }

            // Put zerowidth characters over first fullwidth character cell.
            let line = self.grid.cursor.point.line;
            if self.grid[line][column]
                .shell_flags
                .contains(ShellFlags::WIDE_CHAR_SPACER)
            {
                column.0 = column.saturating_sub(1);
            }

            self.grid[line][column].push_zerowidth(c);
            return;
        }

        // Move cursor to next line.
        if self.grid.cursor.input_needs_wrap {
            self.wrapline();
        }

        // If in insert mode, first shift cells to the right.
        let columns = self.columns();
        if self.mode.contains(TermMode::INSERT) && self.grid.cursor.point.column + width < columns {
            let line = self.grid.cursor.point.line;
            let col = self.grid.cursor.point.column;
            let row = &mut self.grid[line][..];

            for col in (col.0..(columns - width)).rev() {
                row.swap(col + width, col);
            }
        }

        if width == 1 {
            self.write_at_cursor(c);
        } else {
            if self.grid.cursor.point.column + 1 >= columns {
                if self.mode.contains(TermMode::LINE_WRAP) {
                    // Insert placeholder before wide char if glyph does not fit in this row.
                    self.grid
                        .cursor
                        .template
                        .shell_flags
                        .insert(ShellFlags::LEADING_WIDE_CHAR_SPACER);
                    self.write_at_cursor(' ');
                    self.grid
                        .cursor
                        .template
                        .shell_flags
                        .remove(ShellFlags::LEADING_WIDE_CHAR_SPACER);
                    self.wrapline();
                } else {
                    // Prevent out of bounds crash when linewrapping is disabled.
                    self.grid.cursor.input_needs_wrap = true;
                    return;
                }
            }

            // Write full width glyph to current cursor cell.
            self.grid
                .cursor
                .template
                .shell_flags
                .insert(ShellFlags::WIDE_CHAR);
            self.write_at_cursor(c);
            self.grid
                .cursor
                .template
                .shell_flags
                .remove(ShellFlags::WIDE_CHAR);

            // Write spacer to cell following the wide glyph.
            self.grid.cursor.point.column += 1;
            self.grid
                .cursor
                .template
                .shell_flags
                .insert(ShellFlags::WIDE_CHAR_SPACER);
            self.write_at_cursor(' ');
            self.grid
                .cursor
                .template
                .shell_flags
                .remove(ShellFlags::WIDE_CHAR_SPACER);
        }

        if self.grid.cursor.point.column + 1 < columns {
            self.grid.cursor.point.column += 1;
        } else {
            self.grid.cursor.input_needs_wrap = true;
        }

        trace!("Current cursor position: {:?}", self.grid.cursor.point);
    }

    #[inline]
    fn decaln(&mut self) {
        trace!("Decalnning");

        for line in (0..self.screen_lines()).map(Line::from) {
            for column in 0..self.columns() {
                let cell = &mut self.grid[line][Column(column)];
                *cell = Cell::default();
                cell.c = 'E';
            }
        }
    }

    #[inline]
    fn goto(&mut self, line: Line, col: Column) {
        trace!("Going to: line={}, col={}", line, col);
        let (y_offset, max_y) = if self.mode.contains(TermMode::ORIGIN) {
            (self.scroll_region.start, self.scroll_region.end - 1)
        } else {
            (Line(0), self.bottommost_line())
        };

        self.grid.cursor.point.line = max(min(line + y_offset, max_y), Line(0));
        self.grid.cursor.point.column = min(col, self.last_column());
        self.grid.cursor.input_needs_wrap = false;
    }

    #[inline]
    fn goto_line(&mut self, line: Line) {
        trace!("Going to line: {}", line);
        self.goto(line, self.grid.cursor.point.column)
    }

    #[inline]
    fn goto_col(&mut self, col: Column) {
        trace!("Going to column: {}", col);
        self.goto(self.grid.cursor.point.line, col)
    }

    #[inline]
    fn insert_blank(&mut self, count: usize) {
        let cursor = &self.grid.cursor;
        let bg = cursor.template.bg;

        // Ensure inserting within terminal bounds
        let count = min(count, self.columns() - cursor.point.column.0);

        let source = cursor.point.column;
        let destination = cursor.point.column.0 + count;
        let num_cells = self.columns() - destination;

        let line = cursor.point.line;
        let row = &mut self.grid[line][..];

        for offset in (0..num_cells).rev() {
            row.swap(destination + offset, source.0 + offset);
        }

        // Cells were just moved out toward the end of the line;
        // fill in between source and dest with blanks.
        for cell in &mut row[source.0..destination] {
            *cell = bg.into();
        }
    }

    #[inline]
    fn move_up(&mut self, lines: usize) {
        trace!("Moving up: {}", lines);
        self.goto(
            self.grid.cursor.point.line - lines,
            self.grid.cursor.point.column,
        )
    }

    #[inline]
    fn move_down(&mut self, lines: usize) {
        trace!("Moving down: {}", lines);
        self.goto(
            self.grid.cursor.point.line + lines,
            self.grid.cursor.point.column,
        )
    }

    #[inline]
    fn move_forward(&mut self, cols: Column) {
        trace!("Moving forward: {}", cols);
        let last_column = self.last_column();
        self.grid.cursor.point.column = min(self.grid.cursor.point.column + cols, last_column);
        self.grid.cursor.input_needs_wrap = false;
    }

    #[inline]
    fn move_backward(&mut self, cols: Column) {
        trace!("Moving backward: {}", cols);
        self.grid.cursor.point.column =
            Column(self.grid.cursor.point.column.saturating_sub(cols.0));
        self.grid.cursor.input_needs_wrap = false;
    }

    #[inline]
    fn move_down_and_cr(&mut self, lines: usize) {
        trace!("Moving down and cr: {}", lines);
        self.goto(self.grid.cursor.point.line + lines, Column(0))
    }

    #[inline]
    fn move_up_and_cr(&mut self, lines: usize) {
        trace!("Moving up and cr: {}", lines);
        self.goto(self.grid.cursor.point.line - lines, Column(0))
    }

    /// Insert tab at cursor position.
    #[inline]
    fn put_tab(&mut self, mut count: u16) {
        // A tab after the last column is the same as a linebreak.
        if self.grid.cursor.input_needs_wrap {
            self.wrapline();
            return;
        }

        while self.grid.cursor.point.column < self.columns() && count != 0 {
            count -= 1;

            let c = self.grid.cursor.charsets[self.active_charset].map('\t');
            let cell = self.grid.cursor_cell();
            if cell.c == ' ' {
                cell.c = c;
            }

            loop {
                if (self.grid.cursor.point.column + 1) == self.columns() {
                    break;
                }

                self.grid.cursor.point.column += 1;

                if self.tabs[self.grid.cursor.point.column] {
                    break;
                }
            }
        }
    }

    /// Backspace.
    #[inline]
    fn backspace(&mut self) {
        trace!("Backspace");

        if self.grid.cursor.point.column > Column(0) {
            self.grid.cursor.point.column -= 1;
            self.grid.cursor.input_needs_wrap = false;
        }
    }

    /// Carriage return.
    #[inline]
    fn carriage_return(&mut self) {
        trace!("Carriage return");
        self.grid.cursor.point.column = Column(0);
        self.grid.cursor.input_needs_wrap = false;
    }

    /// Linefeed.
    #[inline]
    fn linefeed(&mut self) {
        trace!("Linefeed");
        let next = self.grid.cursor.point.line + 1;
        if next == self.scroll_region.end {
            self.scroll_up(1);
        } else if next < self.screen_lines() {
            self.grid.cursor.point.line += 1;
        }
    }

    /// Set current position as a tabstop.
    #[inline]
    fn bell(&mut self) {
        trace!("Bell");
    }

    #[inline]
    fn substitute(&mut self) {
        trace!("[unimplemented] Substitute");
    }

    /// Run LF/NL.
    ///
    /// LF/NL mode has some interesting history. According to ECMA-48 4th
    /// edition, in LINE FEED mode,
    ///
    /// > The execution of the formatter functions LINE FEED (LF), FORM FEED
    /// (FF), LINE TABULATION (VT) cause only movement of the active position in
    /// the direction of the line progression.
    ///
    /// In NEW LINE mode,
    ///
    /// > The execution of the formatter functions LINE FEED (LF), FORM FEED
    /// (FF), LINE TABULATION (VT) cause movement to the line home position on
    /// the following line, the following form, etc. In the case of LF this is
    /// referred to as the New Line (NL) option.
    ///
    /// Additionally, ECMA-48 4th edition says that this option is deprecated.
    /// ECMA-48 5th edition only mentions this option (without explanation)
    /// saying that it's been removed.
    ///
    /// As an emulator, we need to support it since applications may still rely
    /// on it.
    #[inline]
    fn newline(&mut self) {
        self.linefeed();

        if self.mode.contains(TermMode::LINE_FEED_NEW_LINE) {
            self.carriage_return();
        }
    }

    #[inline]
    fn set_horizontal_tabstop(&mut self) {
        trace!("Setting horizontal tabstop");
        self.tabs[self.grid.cursor.point.column] = true;
    }

    #[inline]
    fn scroll_up(&mut self, lines: usize) {
        let origin = self.scroll_region.start;
        self.scroll_up_relative(origin, lines);
    }

    #[inline]
    fn scroll_down(&mut self, lines: usize) {
        let origin = self.scroll_region.start;

        self.scroll_down_relative(origin, lines);
    }

    #[inline]
    fn insert_blank_lines(&mut self, lines: usize) {
        trace!("Inserting blank {} lines", lines);

        let origin = self.grid.cursor.point.line;
        if self.scroll_region.contains(&origin) {
            self.scroll_down_relative(origin, lines);
        }
    }

    #[inline]
    fn delete_lines(&mut self, lines: usize) {
        let origin = self.grid.cursor.point.line;
        let lines = min(self.screen_lines() - origin.0 as usize, lines);

        trace!("Deleting {} lines", lines);

        if lines > 0 && self.scroll_region.contains(&origin) {
            self.scroll_up_relative(origin, lines);
        }
    }

    #[inline]
    fn erase_chars(&mut self, count: Column) {
        let cursor = &self.grid.cursor;

        trace!(
            "Erasing chars: count={}, col={}",
            count,
            cursor.point.column
        );

        let start = cursor.point.column;
        let end = min(start + count, Column(self.columns()));

        // Cleared cells have current background color set.
        let bg = self.grid.cursor.template.bg;
        let line = cursor.point.line;
        let row = &mut self.grid[line];
        for cell in &mut row[start..end] {
            *cell = bg.into();
        }
    }

    #[inline]
    fn delete_chars(&mut self, count: usize) {
        let columns = self.columns();
        let cursor = &self.grid.cursor;
        let bg = cursor.template.bg;

        // Ensure deleting within terminal bounds.
        let count = min(count, columns);

        let start = cursor.point.column.0;
        let end = min(start + count, columns - 1);
        let num_cells = columns - end;

        let line = cursor.point.line;
        let row = &mut self.grid[line][..];

        for offset in 0..num_cells {
            row.swap(start + offset, end + offset);
        }

        // Clear last `count` cells in the row. If deleting 1 char, need to delete
        // 1 cell.
        let end = columns - count;
        for cell in &mut row[end..] {
            *cell = bg.into();
        }
    }

    #[inline]
    fn move_backward_tabs(&mut self, count: u16) {
        trace!("Moving backward {} tabs", count);

        for _ in 0..count {
            let mut col = self.grid.cursor.point.column;
            for i in (0..(col.0)).rev() {
                if self.tabs[index::Column(i)] {
                    col = index::Column(i);
                    break;
                }
            }
            self.grid.cursor.point.column = col;
        }
    }

    #[inline]
    fn move_forward_tabs(&mut self, count: u16) {
        trace!("[unimplemented] Moving forward {} tabs", count);
    }

    #[inline]
    fn save_cursor_position(&mut self) {
        trace!("Saving cursor position");

        self.grid.saved_cursor = self.grid.cursor.clone();
    }

    #[inline]
    fn restore_cursor_position(&mut self) {
        trace!("Restoring cursor position");

        self.grid.cursor = self.grid.saved_cursor.clone();
    }

    #[inline]
    fn clear_line(&mut self, mode: ansi::LineClearMode) {
        trace!("Clearing line: {:?}", mode);

        let cursor = &self.grid.cursor;
        let bg = cursor.template.bg;

        let point = cursor.point;
        let row = &mut self.grid[point.line];

        match mode {
            ansi::LineClearMode::Right => {
                for cell in &mut row[point.column..] {
                    *cell = bg.into();
                }
            }
            ansi::LineClearMode::Left => {
                for cell in &mut row[..=point.column] {
                    *cell = bg.into();
                }
            }
            ansi::LineClearMode::All => {
                for cell in &mut row[..] {
                    *cell = bg.into();
                }
            }
        }
    }

    /// Set the indexed color value.
    #[inline]
    fn set_color(&mut self, index: usize, color: Rgb) {
        trace!("Setting color[{}] = {:?}", index, color);
        self.colors[index] = Some(color);
    }

    /// Reset the indexed color to original value.
    #[inline]
    fn reset_color(&mut self, index: usize) {
        trace!("Resetting color[{}]", index);
        self.colors[index] = None;
    }

    #[inline]
    fn clear_screen(&mut self, mode: ansi::ClearMode) {
        trace!("Clearing screen: {:?}", mode);
        let bg = self.grid.cursor.template.bg;

        let screen_lines = self.screen_lines();

        match mode {
            ansi::ClearMode::Above => {
                let cursor = self.grid.cursor.point;

                // If clearing more than one line.
                if cursor.line > 1 {
                    // Fully clear all lines before the current line.
                    self.grid.reset_region(..cursor.line);
                }

                // Clear up to the current column in the current line.
                let end = min(cursor.column + 1, Column(self.columns()));
                for cell in &mut self.grid[cursor.line][..end] {
                    *cell = bg.into();
                }
            }
            ansi::ClearMode::Below => {
                let cursor = self.grid.cursor.point;
                for cell in &mut self.grid[cursor.line][cursor.column..] {
                    *cell = bg.into();
                }

                if (cursor.line.0 as usize) < screen_lines - 1 {
                    self.grid.reset_region((cursor.line + 1)..);
                }
            }
            ansi::ClearMode::All => {
                if self.mode.contains(TermMode::ALT_SCREEN) {
                    self.grid.reset_region(..);
                } else {
                    self.grid.clear_viewport();
                }
            }
            ansi::ClearMode::Saved if self.history_size() > 0 => {
                self.grid.clear_history();
            }
            // We have no history to clear.
            ansi::ClearMode::Saved => (),
        }
    }

    #[inline]
    fn clear_tabs(&mut self, mode: ansi::TabulationClearMode) {
        trace!("Clearing tabs: {:?}", mode);
        match mode {
            ansi::TabulationClearMode::Current => {
                self.tabs[self.grid.cursor.point.column] = false;
            }
            ansi::TabulationClearMode::All => {
                self.tabs.clear_all();
            }
        }
    }

    /// Reset all important fields in the term struct.
    #[inline]
    fn reset_state(&mut self) {
        if self.mode.contains(TermMode::ALT_SCREEN) {
            mem::swap(&mut self.grid, &mut self.inactive_grid);
        }
        self.active_charset = Default::default();
        self.grid.reset();
        self.inactive_grid.reset();
        self.scroll_region = Line(0)..Line(self.screen_lines() as i32);
        self.tabs = TabStops::new(self.columns());
        self.title_stack = Vec::new();
        self.title = None;

        // Preserve vi mode across resets.
        self.mode &= TermMode::VI;
        self.mode.insert(TermMode::default());
    }

    #[inline]
    fn reverse_index(&mut self) {
        trace!("Reversing index");
        // If cursor is at the top.
        if self.grid.cursor.point.line == self.scroll_region.start {
            self.scroll_down(1);
        } else {
            self.grid.cursor.point.line = max(self.grid.cursor.point.line - 1, Line(0));
        }
    }

    /// Set a terminal attribute.
    #[inline]
    fn terminal_attribute(&mut self, attr: Attr) {
        trace!("Setting attribute: {:?}", attr);

        let cursor = &mut self.grid.cursor;

        let color_match =
            |color: Color, vtermcolor: fig_color::VTermColor| match (color, vtermcolor) {
                (Color::Named(name), fig_color::VTermColor::Indexed(j)) => {
                    (name as usize % 256) == j as usize
                }
                (Color::Indexed(i), fig_color::VTermColor::Indexed(j)) => i == j,
                (Color::Spec(rgb), fig_color::VTermColor::Rgb(r, g, b)) => {
                    rgb.r == r && rgb.g == g && rgb.b == b
                }
                _ => false,
            };

        macro_rules! set_in_suggestion {
            () => {
                let fg = cursor.template.fg;
                let bg = cursor.template.bg;

                let mut in_suggestion = false;

                if let Some(suggestion_color) =
                    match self.shell_state.get_context().shell.as_deref() {
                        Some("fish") => Some(self.shell_state().fish_suggestion_color.as_ref()),
                        Some("zsh") => Some(self.shell_state().zsh_autosuggestion_color.as_ref()),
                        _ => None,
                    }
                    .flatten()
                {
                    let fg_matches = match suggestion_color.fg() {
                        Some(suggestion_fg) => color_match(fg, suggestion_fg),
                        None => true,
                    };

                    let bg_matches = match suggestion_color.bg() {
                        Some(suggestion_bg) => color_match(bg, suggestion_bg),
                        None => true,
                    };

                    if fg_matches && bg_matches {
                        in_suggestion = true;
                    }
                };

                self.grid
                    .cursor
                    .template
                    .fig_flags
                    .set(FigFlags::IN_SUGGESTION, in_suggestion);
            };
        }

        match attr {
            Attr::Foreground(color) => {
                cursor.template.fg = color;
                set_in_suggestion!();
            }
            Attr::Background(color) => {
                cursor.template.bg = color;
                set_in_suggestion!();
            }
            Attr::Reset => {
                cursor.template.fg = Color::Named(NamedColor::Foreground);
                cursor.template.bg = Color::Named(NamedColor::Background);
                cursor.template.shell_flags = ShellFlags::empty();
                set_in_suggestion!();
            }
            Attr::Reverse => cursor.template.shell_flags.insert(ShellFlags::INVERSE),
            Attr::CancelReverse => cursor.template.shell_flags.remove(ShellFlags::INVERSE),
            Attr::Bold => cursor.template.shell_flags.insert(ShellFlags::BOLD),
            Attr::CancelBold => cursor.template.shell_flags.remove(ShellFlags::BOLD),
            Attr::Dim => cursor.template.shell_flags.insert(ShellFlags::DIM),
            Attr::CancelBoldDim => cursor
                .template
                .shell_flags
                .remove(ShellFlags::BOLD | ShellFlags::DIM),
            Attr::Italic => cursor.template.shell_flags.insert(ShellFlags::ITALIC),
            Attr::CancelItalic => cursor.template.shell_flags.remove(ShellFlags::ITALIC),
            Attr::Underline => {
                cursor
                    .template
                    .shell_flags
                    .remove(ShellFlags::DOUBLE_UNDERLINE);
                cursor.template.shell_flags.insert(ShellFlags::UNDERLINE);
            }
            Attr::DoubleUnderline => {
                cursor.template.shell_flags.remove(ShellFlags::UNDERLINE);
                cursor
                    .template
                    .shell_flags
                    .insert(ShellFlags::DOUBLE_UNDERLINE);
            }
            Attr::CancelUnderline => {
                cursor
                    .template
                    .shell_flags
                    .remove(ShellFlags::UNDERLINE | ShellFlags::DOUBLE_UNDERLINE);
            }
            Attr::Hidden => cursor.template.shell_flags.insert(ShellFlags::HIDDEN),
            Attr::CancelHidden => cursor.template.shell_flags.remove(ShellFlags::HIDDEN),
            Attr::Strike => cursor.template.shell_flags.insert(ShellFlags::STRIKEOUT),
            Attr::CancelStrike => cursor.template.shell_flags.remove(ShellFlags::STRIKEOUT),
            _ => {
                trace!("Term got unhandled attr: {:?}", attr);
            }
        }
    }

    #[inline]
    fn set_mode(&mut self, mode: ansi::Mode) {
        trace!("Setting mode: {:?}", mode);
        match mode {
            ansi::Mode::UrgencyHints => self.mode.insert(TermMode::URGENCY_HINTS),
            ansi::Mode::SwapScreenAndSetRestoreCursor => {
                if !self.mode.contains(TermMode::ALT_SCREEN) {
                    self.swap_alt();
                }
            }
            ansi::Mode::ShowCursor => self.mode.insert(TermMode::SHOW_CURSOR),
            ansi::Mode::CursorKeys => self.mode.insert(TermMode::APP_CURSOR),
            // Mouse protocols are mutually exclusive.
            ansi::Mode::ReportMouseClicks => {
                self.mode.remove(TermMode::MOUSE_MODE);
                self.mode.insert(TermMode::MOUSE_REPORT_CLICK);
            }
            ansi::Mode::ReportCellMouseMotion => {
                self.mode.remove(TermMode::MOUSE_MODE);
                self.mode.insert(TermMode::MOUSE_DRAG);
            }
            ansi::Mode::ReportAllMouseMotion => {
                self.mode.remove(TermMode::MOUSE_MODE);
                self.mode.insert(TermMode::MOUSE_MOTION);
            }
            ansi::Mode::ReportFocusInOut => self.mode.insert(TermMode::FOCUS_IN_OUT),
            ansi::Mode::BracketedPaste => self.mode.insert(TermMode::BRACKETED_PASTE),
            // Mouse encodings are mutually exclusive.
            ansi::Mode::SgrMouse => {
                self.mode.remove(TermMode::UTF8_MOUSE);
                self.mode.insert(TermMode::SGR_MOUSE);
            }
            ansi::Mode::Utf8Mouse => {
                self.mode.remove(TermMode::SGR_MOUSE);
                self.mode.insert(TermMode::UTF8_MOUSE);
            }
            ansi::Mode::AlternateScroll => self.mode.insert(TermMode::ALTERNATE_SCROLL),
            ansi::Mode::LineWrap => self.mode.insert(TermMode::LINE_WRAP),
            ansi::Mode::LineFeedNewLine => self.mode.insert(TermMode::LINE_FEED_NEW_LINE),
            ansi::Mode::Origin => self.mode.insert(TermMode::ORIGIN),
            ansi::Mode::ColumnMode => self.deccolm(),
            ansi::Mode::Insert => self.mode.insert(TermMode::INSERT),
            ansi::Mode::BlinkingCursor => {}
        }
    }

    #[inline]
    fn unset_mode(&mut self, mode: ansi::Mode) {
        trace!("Unsetting mode: {:?}", mode);
        match mode {
            ansi::Mode::UrgencyHints => self.mode.remove(TermMode::URGENCY_HINTS),
            ansi::Mode::SwapScreenAndSetRestoreCursor => {
                if self.mode.contains(TermMode::ALT_SCREEN) {
                    self.swap_alt();
                }
            }
            ansi::Mode::ShowCursor => self.mode.remove(TermMode::SHOW_CURSOR),
            ansi::Mode::CursorKeys => self.mode.remove(TermMode::APP_CURSOR),
            ansi::Mode::ReportMouseClicks => {
                self.mode.remove(TermMode::MOUSE_REPORT_CLICK);
            }
            ansi::Mode::ReportCellMouseMotion => {
                self.mode.remove(TermMode::MOUSE_DRAG);
            }
            ansi::Mode::ReportAllMouseMotion => {
                self.mode.remove(TermMode::MOUSE_MOTION);
            }
            ansi::Mode::ReportFocusInOut => self.mode.remove(TermMode::FOCUS_IN_OUT),
            ansi::Mode::BracketedPaste => self.mode.remove(TermMode::BRACKETED_PASTE),
            ansi::Mode::SgrMouse => self.mode.remove(TermMode::SGR_MOUSE),
            ansi::Mode::Utf8Mouse => self.mode.remove(TermMode::UTF8_MOUSE),
            ansi::Mode::AlternateScroll => self.mode.remove(TermMode::ALTERNATE_SCROLL),
            ansi::Mode::LineWrap => self.mode.remove(TermMode::LINE_WRAP),
            ansi::Mode::LineFeedNewLine => self.mode.remove(TermMode::LINE_FEED_NEW_LINE),
            ansi::Mode::Origin => self.mode.remove(TermMode::ORIGIN),
            ansi::Mode::ColumnMode => self.deccolm(),
            ansi::Mode::Insert => self.mode.remove(TermMode::INSERT),
            ansi::Mode::BlinkingCursor => {}
        }
    }

    #[inline]
    fn set_scrolling_region(&mut self, top: usize, bottom: Option<usize>) {
        // Fallback to the last line as default.
        let bottom = bottom.unwrap_or_else(|| self.screen_lines());

        if top >= bottom {
            debug!("Invalid scrolling region: ({};{})", top, bottom);
            return;
        }

        // Bottom should be included in the range, but range end is not
        // usually included. One option would be to use an inclusive
        // range, but instead we just let the open range end be 1
        // higher.
        let start = Line(top as i32 - 1);
        let end = Line(bottom as i32);

        trace!("Setting scrolling region: ({};{})", start, end);

        let screen_lines = Line(self.screen_lines() as i32);
        self.scroll_region.start = min(start, screen_lines);
        self.scroll_region.end = min(end, screen_lines);
        self.goto(Line(0), Column(0));
    }

    #[inline]
    fn set_keypad_application_mode(&mut self) {
        trace!("Setting keypad application mode");
        self.mode.insert(TermMode::APP_KEYPAD);
    }

    #[inline]
    fn unset_keypad_application_mode(&mut self) {
        trace!("Unsetting keypad application mode");
        self.mode.remove(TermMode::APP_KEYPAD);
    }

    #[inline]
    fn configure_charset(&mut self, index: CharsetIndex, charset: StandardCharset) {
        trace!("Configuring charset {:?} as {:?}", index, charset);
        self.grid.cursor.charsets[index] = charset;
    }

    #[inline]
    fn set_active_charset(&mut self, index: CharsetIndex) {
        trace!("Setting active charset {:?}", index);
        self.active_charset = index;
    }

    #[inline]
    fn set_title(&mut self, title: Option<String>) {
        trace!("Setting title to '{:?}'", title);
        self.title = title
    }

    #[inline]
    fn push_title(&mut self) {
        trace!("Pushing '{:?}' onto title stack", self.title);

        if self.title_stack.len() >= TITLE_STACK_MAX_DEPTH {
            let removed = self.title_stack.remove(0);
            trace!(
                "Removing '{:?}' from bottom of title stack that exceeds its maximum depth",
                removed
            );
        }

        self.title_stack.push(self.title.clone());
    }

    #[inline]
    fn pop_title(&mut self) {
        trace!("Attempting to pop title from stack...");

        if let Some(popped) = self.title_stack.pop() {
            trace!("Title '{:?}' popped from stack", popped);
            self.set_title(popped);
        }
    }

    fn new_cmd(&mut self) {
        trace!("Fig new command");

        self.shell_state.cmd_cursor = Some(self.grid().cursor.point);
        trace!("New command cursor: {:?}", self.shell_state.cmd_cursor);

        // Add work around for emojis
        if let Ok(cursor_offset) = std::env::var("FIG_PROMPT_OFFSET_WORKAROUND") {
            if let Ok(offset) = cursor_offset.parse::<i32>() {
                self.shell_state.cmd_cursor = self.shell_state.cmd_cursor.map(|cursor| Point {
                    column: Column((cursor.column.0 as i32 - offset).max(0) as usize),
                    line: cursor.line,
                });

                trace!(
                    "Command cursor offset by '{}' to {:?}",
                    offset,
                    self.shell_state.cmd_cursor
                );
            }
        }

        self.shell_state.preexec = false;

        self.event_proxy
            .send_event(Event::Prompt, &self.shell_state);
        trace!("Prompt event sent");

        if let Some(command) = &self.shell_state.command_info {
            self.event_proxy
                .send_event(Event::CommandInfo(command), &self.shell_state);
            trace!("Command info event sent");
        }
    }

    fn start_prompt(&mut self) {
        trace!("Fig start prompt");
        self.shell_state.has_seen_prompt = true;

        self.grid
            .cursor
            .template
            .fig_flags
            .insert(FigFlags::IN_PROMPT);
    }

    fn end_prompt(&mut self) {
        trace!("Fig end prompt");
        self.grid
            .cursor
            .template
            .fig_flags
            .remove(FigFlags::IN_PROMPT);
    }

    fn pre_exec(&mut self) {
        trace!("Fig PreExec");

        self.event_proxy
            .send_event(Event::PreExec, &self.shell_state);
        trace!("PreExec event sent");

        let buffer = self
            .get_current_buffer()
            .map(|b| b.buffer.trim().to_owned());

        let context = self.shell_state.get_context();
        self.shell_state.command_info = Some(CommandInfo {
            command: buffer,
            shell: context.shell.clone(),
            pid: context.pid,
            session_id: context.session_id.clone(),
            cwd: env::current_dir().ok(),
            time: std::time::SystemTime::now()
                .duration_since(std::time::SystemTime::UNIX_EPOCH)
                .ok()
                .map(|d| d.as_secs()),
            in_ssh: self.shell_state.in_ssh,
            in_docker: self.shell_state.in_docker,
            hostname: context.hostname.clone(),
            exit_code: None,
        });

        self.shell_state.preexec = true;
    }

    fn dir(&mut self, directory: &std::path::Path) {
        trace!("Fig dir: {:?}", directory.display());
        self.shell_state.get_mut_context().current_working_directory =
            Some(directory.to_path_buf());
        match env::set_current_dir(directory) {
            Ok(_) => {}
            Err(e) => log::error!("Failed to set current dir: {}", e),
        }
    }

    fn exit_code(&mut self, exit_code: i32) {
        trace!("Fig exit code: {}", exit_code);
        if let Some(command) = &mut self.shell_state.command_info {
            command.exit_code = Some(exit_code);
        }
    }

    fn shell(&mut self, shell: &str) {
        let shell = shell.trim().to_owned();
        trace!("Fig shell: {:?}", shell);
        let shell_changed = match &self.shell_state.get_context().shell {
            Some(old_shell) => old_shell.ne(&shell),
            None => true,
        };
        self.shell_state.get_mut_context().shell = Some(shell);
        if shell_changed {
            self.event_proxy
                .send_event(Event::ShellChanged, &self.shell_state);
        }
    }

    fn fish_suggestion_color(&mut self, color: &str) {
        trace!("Fig fish suggestion color: {:?}", color);

        if let Some(color_support) = self.shell_state().color_support {
            self.shell_state.fish_suggestion_color =
                fig_color::parse_suggestion_color_fish(color, color_support);
        }
    }

    fn zsh_suggestion_color(&mut self, color: &str) {
        trace!("Fig zsh suggestion color: {:?}", color);

        if let Some(color_support) = self.shell_state().color_support {
            self.shell_state.zsh_autosuggestion_color =
                fig_color::parse_suggestion_color_zsh_autosuggest(color, color_support);
        }
    }

    fn tty(&mut self, tty: &str) {
        let tty = tty.trim().to_owned();
        trace!("Fig tty: {:?}", tty);
        self.shell_state.get_mut_context().tty = Some(tty);
    }

    fn pid(&mut self, pid: i32) {
        trace!("Fig pid: {}", pid);
        self.shell_state.get_mut_context().pid = Some(pid);
    }

    fn session_id(&mut self, session_id: &str) {
        let session_id = session_id.trim().to_owned();
        trace!("Fig session_id: {:?}", session_id);
        self.shell_state.get_mut_context().session_id = Some(session_id);
    }

    fn docker(&mut self, in_docker: bool) {
        trace!("Fig in_docker: {}", in_docker);
        self.shell_state.in_docker = in_docker;
    }

    fn ssh(&mut self, in_ssh: bool) {
        trace!("Fig in_ssh: {}", in_ssh);
        self.shell_state.in_ssh = in_ssh;
    }

    fn hostname(&mut self, hostname: &str) {
        let hostname = hostname.trim().to_owned();
        trace!("Fig hostname: {:?}", hostname);
        self.shell_state.get_mut_context().hostname = Some(hostname);
    }

    fn log(&mut self, fig_log_level: &str) {
        let fig_log_level = fig_log_level.trim().to_owned();
        trace!("Fig log: {:?}", fig_log_level);

        self.shell_state.fig_log_level = Some(fig_log_level.clone());
        self.event_proxy.log_level_event(Some(fig_log_level));
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ClipboardType {
    Clipboard,
    Selection,
}

struct TabStops {
    tabs: Vec<bool>,
}

impl TabStops {
    #[inline]
    fn new(columns: usize) -> TabStops {
        TabStops {
            tabs: (0..columns).map(|i| i % INITIAL_TABSTOPS == 0).collect(),
        }
    }

    /// Remove all tabstops.
    #[inline]
    fn clear_all(&mut self) {
        unsafe {
            ptr::write_bytes(self.tabs.as_mut_ptr(), 0, self.tabs.len());
        }
    }

    /// Increase tabstop capacity.
    #[inline]
    fn resize(&mut self, columns: usize) {
        let mut index = self.tabs.len();
        self.tabs.resize_with(columns, || {
            let is_tabstop = index % INITIAL_TABSTOPS == 0;
            index += 1;
            is_tabstop
        });
    }
}

impl Index<Column> for TabStops {
    type Output = bool;

    fn index(&self, index: Column) -> &bool {
        &self.tabs[index.0]
    }
}

impl IndexMut<Column> for TabStops {
    fn index_mut(&mut self, index: Column) -> &mut bool {
        self.tabs.index_mut(index.0)
    }
}

/// Terminal cursor rendering information.
#[derive(Copy, Clone)]
pub struct RenderableCursor {
    pub point: Point,
}

impl RenderableCursor {
    fn new<T>(term: &Term<T>) -> Self {
        // Cursor position.
        let mut point = term.grid.cursor.point;
        if term.grid[point]
            .shell_flags
            .contains(ShellFlags::WIDE_CHAR_SPACER)
        {
            point.column -= 1;
        }

        Self { point }
    }
}

/// Visible terminal content.
///
/// This contains all content required to render the current terminal view.
pub struct RenderableContent<'a> {
    pub display_iter: GridIterator<'a, Cell>,
    pub cursor: RenderableCursor,
    pub display_offset: usize,
    pub colors: &'a color::Colors,
    pub mode: TermMode,
}

impl<'a> RenderableContent<'a> {
    fn new<T>(term: &'a Term<T>) -> Self {
        Self {
            display_iter: term.grid().display_iter(),
            display_offset: term.grid().display_offset(),
            cursor: RenderableCursor::new(term),
            colors: &term.colors,
            mode: *term.mode(),
        }
    }
}

/// Terminal test helpers.
pub mod test {
    use super::*;

    use unicode_width::UnicodeWidthChar;

    use crate::index::Column;

    /// Construct a terminal from its content as string.
    ///
    /// A `\n` will break line and `\r\n` will break line without wrapping.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use alacritty_terminal::term::test::mock_term;
    ///
    /// // Create a terminal with the following cells:
    /// //
    /// // [h][e][l][l][o] <- WRAPLINE flag set
    /// // [:][)][ ][ ][ ]
    /// // [t][e][s][t][ ]
    /// mock_term(
    ///     "\
    ///     hello\n:)\r\ntest",
    /// );
    /// ```
    pub fn mock_term(content: &str) -> Term<()> {
        let lines: Vec<&str> = content.split('\n').collect();
        let num_cols = lines
            .iter()
            .map(|line| {
                line.chars()
                    .filter(|c| *c != '\r')
                    .map(|c| c.width().unwrap())
                    .sum()
            })
            .max()
            .unwrap_or(0);

        // Create terminal with the appropriate dimensions.
        let size = SizeInfo::new(lines.len(), num_cols);
        let mut term = Term::new(size, (), 100);

        // Fill terminal with content.
        for (line, text) in lines.iter().enumerate() {
            let line = Line(line as i32);
            if !text.ends_with('\r') && line + 1 != lines.len() {
                term.grid[line][Column(num_cols - 1)]
                    .shell_flags
                    .insert(ShellFlags::WRAPLINE);
            }

            let mut index = 0;
            for c in text.chars().take_while(|c| *c != '\r') {
                term.grid[line][Column(index)].c = c;

                // Handle fullwidth characters.
                let width = c.width().unwrap();
                if width == 2 {
                    term.grid[line][Column(index)]
                        .shell_flags
                        .insert(ShellFlags::WIDE_CHAR);
                    term.grid[line][Column(index + 1)]
                        .shell_flags
                        .insert(ShellFlags::WIDE_CHAR_SPACER);
                }

                index += width;
            }
        }

        term
    }
}

// #[cfg(test)]
// mod tests {
//     use super::*;

//     use crate::ansi::{self, CharsetIndex, Handler, StandardCharset};
//     use crate::grid::Scroll;
//     use crate::index::{Column, Point};

//     #[test]
//     fn scroll_display_page_up() {
//         let size = SizeInfo::new(5., 10., 1.0, 1.0, 0.0, 0.0, false);
//         let mut term = Term::new(size, (), 10_000);

//         // Create 11 lines of scrollback.
//         for _ in 0..20 {
//             term.newline();
//         }

//         // Scrollable amount to top is 11.
//         term.scroll_display(Scroll::PageUp);
//         assert_eq!(term.grid.display_offset(), 10);

//         // Scrollable amount to top is 1.
//         term.scroll_display(Scroll::PageUp);
//         assert_eq!(term.grid.display_offset(), 11);

//         // Scrollable amount to top is 0.
//         term.scroll_display(Scroll::PageUp);
//         assert_eq!(term.grid.display_offset(), 11);
//     }

//     #[test]
//     fn scroll_display_page_down() {
//         let size = SizeInfo::new(5., 10., 1.0, 1.0, 0.0, 0.0, false);
//         let mut term = Term::new(size, (), 10_000);

//         // Create 11 lines of scrollback.
//         for _ in 0..20 {
//             term.newline();
//         }

//         // Change display_offset to topmost.
//         term.grid_mut().scroll_display(Scroll::Top);

//         // Scrollable amount to bottom is 11.
//         term.scroll_display(Scroll::PageDown);
//         assert_eq!(term.grid.display_offset(), 1);

//         // Scrollable amount to bottom is 1.
//         term.scroll_display(Scroll::PageDown);
//         assert_eq!(term.grid.display_offset(), 0);

//         // Scrollable amount to bottom is 0.
//         term.scroll_display(Scroll::PageDown);
//         assert_eq!(term.grid.display_offset(), 0);
//     }

//     #[test]
//     fn input_line_drawing_character() {
//         let size = SizeInfo::new(21.0, 51.0, 3.0, 3.0, 0.0, 0.0, false);
//         let mut term = Term::new(size, (), 10_000);
//         let cursor = Point::new(Line(0), Column(0));
//         term.configure_charset(
//             CharsetIndex::G0,
//             StandardCharset::SpecialCharacterAndLineDrawing,
//         );
//         term.input('a');

//         assert_eq!(term.grid()[cursor].c, '▒');
//     }

//     #[test]
//     fn clear_viewport_set_vi_cursor_into_viewport() {
//         let size = SizeInfo::new(10.0, 20.0, 1.0, 1.0, 0.0, 0.0, false);
//         let mut term = Term::new(size, (), 10_000);

//         // Create 10 lines of scrollback.
//         for _ in 0..29 {
//             term.newline();
//         }

//         // Change the display area and the vi cursor position.
//         term.scroll_display(Scroll::Top);

//         // Clear the viewport.
//         term.clear_screen(ansi::ClearMode::All);

//         assert_eq!(term.grid.display_offset(), 0);
//     }

//     #[test]
//     fn clear_scrollback_set_vi_cursor_into_viewport() {
//         let size = SizeInfo::new(10.0, 20.0, 1.0, 1.0, 0.0, 0.0, false);
//         let mut term = Term::new(size, (), 10_000);

//         // Create 10 lines of scrollback.
//         for _ in 0..29 {
//             term.newline();
//         }

//         // Change the display area and the vi cursor position.
//         term.scroll_display(Scroll::Top);

//         // Clear the scrollback buffer.
//         term.clear_screen(ansi::ClearMode::Saved);

//         assert_eq!(term.grid.display_offset(), 0);
//     }

//     #[test]
//     fn clear_saved_lines() {
//         let size = SizeInfo::new(21.0, 51.0, 3.0, 3.0, 0.0, 0.0, false);
//         let mut term = Term::new(size, (), 10_000);

//         // Add one line of scrollback.
//         term.grid.scroll_up(&(Line(0)..Line(1)), 1);

//         // Clear the history.
//         term.clear_screen(ansi::ClearMode::Saved);

//         // Make sure that scrolling does not change the grid.
//         let mut scrolled_grid = term.grid.clone();
//         scrolled_grid.scroll_display(Scroll::Top);

//         // Truncate grids for comparison.
//         scrolled_grid.truncate();
//         term.grid.truncate();

//         assert_eq!(term.grid, scrolled_grid);
//     }

//     #[test]
//     fn grow_lines_updates_active_cursor_pos() {
//         let mut size = SizeInfo::new(100.0, 10.0, 1.0, 1.0, 0.0, 0.0, false);
//         let mut term = Term::new(size, (), 10_000);

//         // Create 10 lines of scrollback.
//         for _ in 0..19 {
//             term.newline();
//         }
//         assert_eq!(term.history_size(), 10);
//         assert_eq!(term.grid.cursor.point, Point::new(Line(9), Column(0)));

//         // Increase visible lines.
//         size.screen_lines = 30;
//         term.resize(size);

//         assert_eq!(term.history_size(), 0);
//         assert_eq!(term.grid.cursor.point, Point::new(Line(19), Column(0)));
//     }

//     #[test]
//     fn grow_lines_updates_inactive_cursor_pos() {
//         let mut size = SizeInfo::new(100.0, 10.0, 1.0, 1.0, 0.0, 0.0, false);
//         let mut term = Term::new(size, (), 10_000);

//         // Create 10 lines of scrollback.
//         for _ in 0..19 {
//             term.newline();
//         }
//         assert_eq!(term.history_size(), 10);
//         assert_eq!(term.grid.cursor.point, Point::new(Line(9), Column(0)));

//         // Enter alt screen.
//         term.set_mode(ansi::Mode::SwapScreenAndSetRestoreCursor);

//         // Increase visible lines.
//         size.screen_lines = 30;
//         term.resize(size);

//         // Leave alt screen.
//         term.unset_mode(ansi::Mode::SwapScreenAndSetRestoreCursor);

//         assert_eq!(term.history_size(), 0);
//         assert_eq!(term.grid.cursor.point, Point::new(Line(19), Column(0)));
//     }

//     #[test]
//     fn shrink_lines_updates_active_cursor_pos() {
//         let mut size = SizeInfo::new(100.0, 10.0, 1.0, 1.0, 0.0, 0.0, false);
//         let mut term = Term::new(size, (), 10_000);

//         // Create 10 lines of scrollback.
//         for _ in 0..19 {
//             term.newline();
//         }
//         assert_eq!(term.history_size(), 10);
//         assert_eq!(term.grid.cursor.point, Point::new(Line(9), Column(0)));

//         // Increase visible lines.
//         size.screen_lines = 5;
//         term.resize(size);

//         assert_eq!(term.history_size(), 15);
//         assert_eq!(term.grid.cursor.point, Point::new(Line(4), Column(0)));
//     }

//     #[test]
//     fn shrink_lines_updates_inactive_cursor_pos() {
//         let mut size = SizeInfo::new(100.0, 10.0, 1.0, 1.0, 0.0, 0.0, false);
//         let mut term = Term::new(size, (), 10_000);

//         // Create 10 lines of scrollback.
//         for _ in 0..19 {
//             term.newline();
//         }
//         assert_eq!(term.history_size(), 10);
//         assert_eq!(term.grid.cursor.point, Point::new(Line(9), Column(0)));

//         // Enter alt screen.
//         term.set_mode(ansi::Mode::SwapScreenAndSetRestoreCursor);

//         // Increase visible lines.
//         size.screen_lines = 5;
//         term.resize(size);

//         // Leave alt screen.
//         term.unset_mode(ansi::Mode::SwapScreenAndSetRestoreCursor);

//         assert_eq!(term.history_size(), 15);
//         assert_eq!(term.grid.cursor.point, Point::new(Line(4), Column(0)));
//     }

//     #[test]
//     fn window_title() {
//         let size = SizeInfo::new(21.0, 51.0, 3.0, 3.0, 0.0, 0.0, false);
//         let mut term = Term::new(size, (), 10_000);

//         // Title None by default.
//         assert_eq!(term.title, None);

//         // Title can be set.
//         term.set_title(Some("Test".into()));
//         assert_eq!(term.title, Some("Test".into()));

//         // Title can be pushed onto stack.
//         term.push_title();
//         term.set_title(Some("Next".into()));
//         assert_eq!(term.title, Some("Next".into()));
//         assert_eq!(term.title_stack.get(0).unwrap(), &Some("Test".into()));

//         // Title can be popped from stack and set as the window title.
//         term.pop_title();
//         assert_eq!(term.title, Some("Test".into()));
//         assert!(term.title_stack.is_empty());

//         // Title stack doesn't grow infinitely.
//         for _ in 0..4097 {
//             term.push_title();
//         }
//         assert_eq!(term.title_stack.len(), 4096);

//         // Title and title stack reset when terminal state is reset.
//         term.push_title();
//         term.reset_state();
//         assert_eq!(term.title, None);
//         assert!(term.title_stack.is_empty());

//         // Title stack pops back to default.
//         term.title = None;
//         term.push_title();
//         term.set_title(Some("Test".into()));
//         term.pop_title();
//         assert_eq!(term.title, None);

//         // Title can be reset to default.
//         term.title = Some("Test".into());
//         term.set_title(None);
//         assert_eq!(term.title, None);
//     }
// }
