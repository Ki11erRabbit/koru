use crate::kernel::buffer::TextBufferImpl;

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
    }
}


#[derive(Copy, Clone)]
pub struct Cursor {
    /// This cursor may not be aligned to a line
    logical_cursor: GridCursor,
    /// This cursor will be aligned to a line
    real_cursor: GridCursor,
    /// This is where the mark is currently located
    mark: Option<GridCursor>,
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
    
    pub fn place_mark(&mut self) {
        self.mark = Some(self.real_cursor);
    }
    
    pub fn remove_mark(&mut self) {
        self.mark = None;
    }
    
    pub fn flip_mark(&mut self) {
        if let Some(mark) = self.mark.as_mut() {
            std::mem::swap(&mut self.real_cursor, mark);
            self.logical_cursor = self.real_cursor;
        }
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



#[derive(Copy, Clone, PartialEq, Eq)]
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

impl Default for GridCursor {
    fn default() -> Self {
        GridCursor::new(0, 0)
    }
}
