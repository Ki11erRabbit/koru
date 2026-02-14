use std::sync::Arc;
use scheme_rs::exceptions::Exception;
use scheme_rs::gc::Trace;
use scheme_rs::records::{rtd, Record, RecordTypeDescriptor, SchemeCompatible};
use scheme_rs::registry::bridge;
use scheme_rs::value::Value;
use crate::kernel::buffer::TextBufferImpl;

#[derive(Clone, Debug, Trace)]
pub struct Cursors {
    pub cursors: Vec<Cursor>
}

impl SchemeCompatible for Cursors {
    fn rtd() -> Arc<RecordTypeDescriptor>
    where
        Self: Sized
    {
        rtd!(name: "&Cursors", sealed: true)
    }
}


#[bridge(name = "cursors-create", lib = "(koru-cursor)")]
pub fn cursors_create() -> Result<Vec<Value>, Exception> {
    let cursors = Cursors {
        cursors: Vec::new(),
    };
    let value = Record::from_rust_type(cursors);
    Ok(vec![Value::from(value)])
}

#[derive(Debug, Copy, Clone, Trace)]
pub enum CursorMark {
    None,
    Point,
    Line,
    Box,
    File
}

#[derive(Copy, Clone)]
pub enum CursorDirection {
    Up,
    Down,
    Left {
        /// Whether to go to the previous line or not.
        wrap: bool,
    },
    Right {
        /// Whether to go to the next line or not.
        wrap: bool,
    },
    LineStart,
    LineEnd,
    BufferStart,
    BufferEnd,
}


#[derive(Debug, Copy, Clone, Trace)]
pub struct Cursor {
    /// This cursor may not be aligned to a line
    logical_cursor: GridCursor,
    /// This cursor will be aligned to a line
    real_cursor: GridCursor,
    /// This is where the mark is currently located
    pub mark: Option<GridCursor>,
    /// The state that the mark is currently in
    pub mark_state: CursorMark,
    main_cursor: bool,
}

impl Cursor {
    pub fn new(
        logical_cursor: GridCursor,
    ) -> Self {
        Self {
            logical_cursor,
            real_cursor: logical_cursor,
            mark: None,
            main_cursor: false,
            mark_state: CursorMark::None,
        }
    }

    pub fn new_main(
        logical_cursor: GridCursor,
    ) -> Self {
        Self {
            logical_cursor,
            real_cursor: logical_cursor,
            mark: None,
            main_cursor: true,
            mark_state: CursorMark::None,
        }
    }

    pub fn is_main(&self) -> bool {
        self.main_cursor
    }

    pub fn line(&self) -> usize {
        self.real_cursor.line
    }

    pub fn column(&self) -> usize {
        self.real_cursor.column
    }

    pub fn mark_line(&self) -> Option<usize> {
        self.mark.map(|c| c.line)
    }

    pub fn mark_column(&self) -> Option<usize> {
        self.mark.map(|c| c.column)
    }

    pub fn is_mark_set(&self) -> bool {
        self.mark.is_some()
    }

    pub fn is_mark_and_cursor_same(&self) -> bool {
        if let Some(mark) = self.mark {
            mark == self.real_cursor
        } else {
            false
        }
    }

    pub fn set_column(&mut self, column: usize) {
        self.real_cursor.column = column;
        self.logical_cursor.column = column;
    }

    pub fn set_line(&mut self, line: usize) {
        self.logical_cursor.line = line;
        self.real_cursor.line = line;
    }

    pub fn at_line_start(&self) -> bool {
        self.real_cursor.column == 0
    }

    pub fn at_line_end(&self, buffer: &dyn TextBufferImpl) -> bool {
        let line_len = buffer.line_length(self.line());
        self.logical_cursor.column == line_len
    }

    pub fn move_up(&mut self, buffer: &dyn TextBufferImpl) {
        self.logical_cursor.line = self.logical_cursor.line.saturating_sub(1);
        self.real_cursor.line = self.logical_cursor.line;
        self.real_cursor.column = self.logical_cursor.column;

        let line_len = buffer.line_length(self.line());
        if line_len < self.real_cursor.column {
            self.real_cursor.column = line_len;
        }

    }

    pub fn move_down(&mut self, buffer: &dyn TextBufferImpl) {
        if buffer.is_there_next_line(self.line()) {
            self.logical_cursor.line = self.logical_cursor.line + 1;
            self.real_cursor.line = self.logical_cursor.line;
            self.real_cursor.column = self.logical_cursor.column;

            let line_len = buffer.line_length(self.line());
            if line_len < self.real_cursor.column {
                self.real_cursor.column = line_len;
            }
        }
    }

    pub fn move_left(&mut self, line_len: usize) {
        if self.real_cursor.column > line_len {
            self.real_cursor.column = line_len;
            self.logical_cursor.column = line_len;
        }

        self.real_cursor.column = self.real_cursor.column.saturating_sub(1);
        self.logical_cursor.column = self.real_cursor.column;
    }

    pub fn move_right(&mut self, line_len: usize) {
        self.logical_cursor.column = self.logical_cursor.column.saturating_add(1);
        self.real_cursor.column = self.logical_cursor.column;


        // Make real cursor be as long as the line
        if self.real_cursor.column > line_len {
            self.real_cursor.column = line_len;
            self.logical_cursor.column = line_len;
        }
    }

    pub fn place_point_mark(&mut self) {
        self.mark = Some(self.real_cursor);
        self.mark_state = CursorMark::Point;
    }

    pub fn place_line_mark(&mut self) {
        self.mark = Some(self.real_cursor);
        self.mark_state = CursorMark::Line;
    }

    pub fn place_box_mark(&mut self) {
        self.mark = Some(self.real_cursor);
        self.mark_state = CursorMark::Box;
    }

    pub fn place_file_mark(&mut self) {
        self.mark_state = CursorMark::File;
    }

    pub fn remove_mark(&mut self) {
        self.mark = None;
        self.mark_state = CursorMark::None;
    }

    pub fn flip_mark(&mut self) {
        if let Some(mark) = self.mark.as_mut() {
            std::mem::swap(&mut self.real_cursor, mark);
            self.logical_cursor = self.real_cursor;
        }
    }

    pub fn unset_main(&mut self) {
        self.main_cursor = false;
    }

    pub fn set_main(&mut self) {
        self.main_cursor = true;
    }

    /// This function checks if the provided col and row are at the end of a mark.
    pub fn is_mark_active(&self, col: usize, row: usize) -> bool {
        match self.mark_state {
            CursorMark::None => false,
            CursorMark::Point | CursorMark::Box => {
                if let Some(mark) = self.mark {
                    mark.column == col && mark.line == row
                } else {
                    false
                }
            }
            CursorMark::Line => {
                if let Some(mark) = self.mark {
                    mark.line == row
                } else {
                    false
                }
            }
            CursorMark::File => {
                true
            }
        }
    }

    /// This function checks if the provided col and row are at the start of a mark.
    pub fn is_point_active(&self, col: usize, row: usize) -> bool {
        match self.mark_state {
            CursorMark::None => false,
            CursorMark::Point | CursorMark::Box => {
                self.real_cursor.column == col && self.real_cursor.line == row
            }
            CursorMark::Line => {
                self.real_cursor.line == row
            }
            CursorMark::File => {
                self.real_cursor.line == row && self.real_cursor.column == col
            }
        }
    }

    pub fn at_cursor(&self, col: usize, row: usize) -> bool {
        self.real_cursor.column == col && self.real_cursor.line == row
    }
}

impl SchemeCompatible for Cursor {
    fn rtd() -> Arc<RecordTypeDescriptor>
    where
        Self: Sized
    {
        rtd!(name: "&Cursor", sealed: true)
    }
}

impl Default for Cursor {
    fn default() -> Self {
        Cursor::new(
            GridCursor::new(0, 0),
        )
    }
}

impl PartialEq for Cursor {
    fn eq(&self, other: &Cursor) -> bool {
        self.real_cursor == other.real_cursor && self.logical_cursor == other.logical_cursor
    }
}



#[derive(Debug, Copy, Clone, PartialEq, Eq, Trace)]
pub struct GridCursor {
    pub line: usize,
    pub column: usize,
}

impl GridCursor {
    pub fn new(line: usize, column: usize) -> Self {
        Self {
            line,
            column,
        }
    }
}

impl SchemeCompatible for GridCursor {
    fn rtd() -> Arc<RecordTypeDescriptor>
    where
        Self: Sized
    {
        rtd!(name: "&GridCursor", sealed: true)
    }
}

impl Default for GridCursor {
    fn default() -> Self {
        GridCursor::new(0, 0)
    }
}
