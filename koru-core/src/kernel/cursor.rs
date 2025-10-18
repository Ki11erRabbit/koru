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


#[derive(Copy, Clone, PartialEq, Eq)]
pub enum LeadingEdge {
    Start,
    End
}

#[derive(Copy, Clone)]
pub struct Cursor {
    /// This cursor may not be aligned to a line
    pub logical_cursor: LogicalCursor,
    /// This cursor will be aligned to a line
    pub byte_cursor: ByteCursor,
    /// Where the cursor is located on the selection
    pub leading_edge: LeadingEdge,
    main_cursor: bool,
}

impl Cursor {
    pub fn new(
        logical_cursor: LogicalCursor, 
        byte_cursor: ByteCursor,
        leading_edge: LeadingEdge,
    ) -> Self {
        Self {
            logical_cursor,
            byte_cursor,
            leading_edge,
            main_cursor: false,
        }
    }

    pub fn new_main(
        logical_cursor: LogicalCursor,
        byte_cursor: ByteCursor,
        leading_edge: LeadingEdge,
    ) -> Self {
        Self {
            logical_cursor,
            byte_cursor,
            leading_edge,
            main_cursor: true,
        }
    }
    
    pub fn is_main(&self) -> bool {
        self.main_cursor
    }

    pub fn byte_edge(&self) -> usize {
        match self.leading_edge {
            LeadingEdge::Start => self.byte_cursor.byte_start,
            LeadingEdge::End => self.byte_cursor.byte_end,
        }
    }

    pub fn at_line_start(&self) -> bool {
        match self.leading_edge {
            LeadingEdge::Start => {
                self.logical_cursor.column_start == 0
            }
            LeadingEdge::End => {
                self.logical_cursor.column_end == 0
            }
        }
    }
    
    pub fn at_line_end(&self, buffer: &dyn TextBufferImpl) -> bool {
        match self.leading_edge {
            LeadingEdge::Start => {
                let line_len = buffer.line_length(self.byte_cursor.byte_start);
                self.logical_cursor.column_start == line_len - 1
            }
            LeadingEdge::End => {
                let line_len = buffer.line_length(self.byte_cursor.byte_end);
                self.logical_cursor.column_end == line_len
            }
        }
    }
    
    pub fn move_logical_up(&mut self) {
        match self.leading_edge {
            LeadingEdge::Start => {
                self.logical_cursor.line_start = self.logical_cursor.line_start.saturating_sub(1);
                self.logical_cursor.line_end = self.logical_cursor.line_start;
            }
            LeadingEdge::End => {
                self.logical_cursor.line_end = self.logical_cursor.line_end.saturating_sub(1);
                self.logical_cursor.line_start = self.logical_cursor.line_end;
            }
        }
    }
    
    pub fn move_logical_down(&mut self, buffer: &dyn TextBufferImpl) {
        match self.leading_edge {
            LeadingEdge::Start => {
                if buffer.is_there_next_line(self.byte_cursor.byte_start) {
                    self.logical_cursor.line_start = self.logical_cursor.line_start.saturating_add(1);
                    self.logical_cursor.line_end = self.logical_cursor.line_start;
                } else {
                    self.logical_cursor.line_end = self.logical_cursor.line_start;
                }
            }
            LeadingEdge::End => {
                if buffer.is_there_next_line(self.byte_cursor.byte_end) {
                    self.logical_cursor.line_end = self.logical_cursor.line_end.saturating_add(1);
                    self.logical_cursor.line_start = self.logical_cursor.line_end;
                } else {
                    self.logical_cursor.line_start = self.logical_cursor.line_end;
                }
            }
        }
    }
    
    pub fn move_left(&mut self) {
        match self.leading_edge {
            LeadingEdge::Start => {
                self.logical_cursor.column_start = self.logical_cursor.column_start.saturating_sub(1);
                self.logical_cursor.column_end = self.logical_cursor.column_start + 1;
            }
            LeadingEdge::End => {
                self.logical_cursor.column_end = self.logical_cursor.column_end.saturating_sub(1);
                if self.logical_cursor.column_end == 0 {
                    self.logical_cursor.column_start = 0;
                    self.logical_cursor.column_end = 1;
                } else {
                    self.logical_cursor.column_start = self.logical_cursor.column_end - 1;
                }
            }
        }
    }
    
    pub fn move_right(&mut self) {
        match self.leading_edge {
            LeadingEdge::Start => {
                self.logical_cursor.column_start = self.logical_cursor.column_start.saturating_add(1);
                self.logical_cursor.column_end = self.logical_cursor.column_start + 1;
            }
            LeadingEdge::End => {
                self.logical_cursor.column_end = self.logical_cursor.column_end.saturating_add(1);
                self.logical_cursor.column_start = self.logical_cursor.column_end - 1;
            }
        }
    }
}

impl PartialEq for Cursor {
    fn eq(&self, other: &Cursor) -> bool {
        self.logical_cursor == other.logical_cursor && self.byte_cursor == other.byte_cursor && self.leading_edge == other.leading_edge
    }
}

#[derive(Copy, Clone, PartialEq, Eq)]
pub struct LogicalCursor {
    pub line_start: usize,
    pub line_end: usize,
    pub column_start: usize,
    pub column_end: usize,
}

impl LogicalCursor {
    pub fn new(line_start: usize, line_end: usize, column_start: usize, column_end: usize) -> Self {
        Self {
            line_end,
            line_start,
            column_end,
            column_start
        }
    }
}


#[derive(Copy, Clone, PartialEq, Eq)]
pub struct ByteCursor {
    pub byte_start: usize,
    pub byte_end: usize,
}

impl ByteCursor {
    pub fn new(byte_start: usize, byte_end: usize) -> Self {
        Self { byte_start, byte_end }
    }
}