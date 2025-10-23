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
    logical_cursor: GridCursor,
    /// This cursor will be aligned to a line
    real_cursor: GridCursor,
    /// Where the cursor is located on the selection
    pub leading_edge: LeadingEdge,
    main_cursor: bool,
}

impl Cursor {
    pub fn new(
        logical_cursor: GridCursor,
        leading_edge: LeadingEdge,
    ) -> Self {
        Self {
            logical_cursor,
            real_cursor: logical_cursor,
            leading_edge,
            main_cursor: false,
        }
    }

    pub fn new_main(
        logical_cursor: GridCursor,
        leading_edge: LeadingEdge,
    ) -> Self {
        Self {
            logical_cursor,
            real_cursor: logical_cursor,
            leading_edge,
            main_cursor: true,
        }
    }

    pub fn is_main(&self) -> bool {
        self.main_cursor
    }

    pub fn line(&self) -> usize {
        match self.leading_edge {
            LeadingEdge::Start => self.real_cursor.line_start,
            LeadingEdge::End => self.real_cursor.line_end,
        }
    }

    pub fn column(&self) -> usize {
        match self.leading_edge {
            LeadingEdge::Start => self.real_cursor.column_start,
            LeadingEdge::End => self.real_cursor.column_end,
        }
    }

    pub fn column_start(&self) -> usize {
        self.real_cursor.column_start
    }
    pub fn column_end(&self) -> usize {
        self.real_cursor.column_end
    }

    pub fn line_start(&self) -> usize {
        self.real_cursor.line_start
    }
    pub fn line_end(&self) -> usize {
        self.real_cursor.line_end
    }

    pub fn set_column_start(&mut self, column: usize) {
        self.real_cursor.column_start = column;
        self.logical_cursor.column_start = column;
    }

    pub fn set_column_end(&mut self, column: usize) {
        self.real_cursor.column_end = column;
        self.logical_cursor.column_end = column;
    }

    pub fn set_line_start(&mut self, line: usize) {
        self.logical_cursor.line_start = line;
        self.real_cursor.line_start = line;
    }

    pub fn set_line_end(&mut self, line: usize) {
        self.logical_cursor.line_end = line;
        self.real_cursor.line_end = line;
    }

    pub fn logical_column_start(&self) -> usize {
        self.logical_cursor.column_start
    }
    pub fn logical_column_end(&self) -> usize {
        self.logical_cursor.column_end
    }

    pub fn at_line_start(&self) -> bool {
        match self.leading_edge {
            LeadingEdge::Start => {
                self.real_cursor.column_start == 0
            }
            LeadingEdge::End => {
                self.real_cursor.column_end.saturating_sub(1) == 0
            }
        }
    }

    pub fn at_line_end(&self, buffer: &dyn TextBufferImpl) -> bool {
        match self.leading_edge {
            LeadingEdge::Start => {
                let line_len = buffer.line_length(self.line());
                self.logical_cursor.column_start == line_len
            }
            LeadingEdge::End => {
                let line_len = buffer.line_length(self.line());
                self.logical_cursor.column_end == line_len
            }
        }
    }

    pub fn move_up(&mut self, buffer: &dyn TextBufferImpl) {
        match self.leading_edge {
            LeadingEdge::Start => {
                // Adjust cursor column
                self.logical_cursor.column_end = self.logical_cursor.column_start + 1;
                self.real_cursor.column_start = self.logical_cursor.column_start;
                self.real_cursor.column_end = self.logical_cursor.column_start + 1;

                self.logical_cursor.line_start = self.logical_cursor.line_start.saturating_sub(1);
                self.logical_cursor.line_end = self.logical_cursor.line_start;
                self.real_cursor.line_start = self.logical_cursor.line_start;
                self.real_cursor.line_end = self.logical_cursor.line_end;

                let line_len = buffer.line_length(self.line());
                if line_len < self.real_cursor.column_start {
                    self.real_cursor.column_start = line_len;
                    self.real_cursor.column_end = self.real_cursor.column_start + 1;
                }
            }
            LeadingEdge::End => {
                // Adjust cursor column
                self.logical_cursor.column_start = self.logical_cursor.column_end.saturating_sub(1);
                self.real_cursor.column_end = self.logical_cursor.column_end;
                self.real_cursor.column_start = self.logical_cursor.column_end.saturating_sub(1);

                self.logical_cursor.line_end = self.logical_cursor.line_end.saturating_sub(1);
                self.logical_cursor.line_start = self.logical_cursor.line_end.saturating_sub(1);
                self.real_cursor.line_start = self.logical_cursor.line_start;
                self.real_cursor.line_end = self.logical_cursor.line_end;

                let line_len = buffer.line_length(self.line());
                if line_len == 0 {
                    self.real_cursor.column_start = 0;
                    self.real_cursor.column_end = 1;
                } else if line_len < self.real_cursor.column_start {
                    self.real_cursor.column_end = line_len;
                    self.real_cursor.column_start = self.real_cursor.column_end - 1;
                }
            }
        }
    }

    pub fn move_down(&mut self, buffer: &dyn TextBufferImpl) {
        match self.leading_edge {
            LeadingEdge::Start => {
                // Adjust cursor column
                self.logical_cursor.column_end = self.logical_cursor.column_start + 1;
                self.real_cursor.column_start = self.logical_cursor.column_start;
                self.real_cursor.column_end = self.logical_cursor.column_start + 1;
                
                if buffer.is_there_next_line(self.line()) {
                    self.logical_cursor.line_start = self.logical_cursor.line_start + 1;
                    self.logical_cursor.line_end = self.logical_cursor.line_start;
                    self.real_cursor.line_start = self.logical_cursor.line_start;
                    self.real_cursor.line_end = self.logical_cursor.line_end;

                    let line_len = buffer.line_length(self.line());
                    if line_len < self.real_cursor.column_start {
                        self.real_cursor.column_start = line_len;
                        self.real_cursor.column_end = self.real_cursor.column_start + 1;
                    }
                }
            }
            LeadingEdge::End => {
                // Adjust cursor column
                self.logical_cursor.column_start = self.logical_cursor.column_end.saturating_sub(1);
                self.real_cursor.column_end = self.logical_cursor.column_end;
                self.real_cursor.column_start = self.logical_cursor.column_end.saturating_sub(1);
                
                if buffer.is_there_next_line(self.line()) {
                    self.logical_cursor.line_end = self.logical_cursor.line_end + 1;
                    self.logical_cursor.line_start = self.logical_cursor.line_end;
                    self.real_cursor.line_start = self.logical_cursor.line_start;
                    self.real_cursor.line_end = self.logical_cursor.line_end;

                    let line_len = buffer.line_length(self.line());
                    if line_len < self.real_cursor.column_start {
                        self.real_cursor.column_start = line_len;
                        self.real_cursor.column_end = self.real_cursor.column_start + 1;
                    }
                }
            }
        }
    }

    pub fn move_left(&mut self, line_len: usize) {
        match self.leading_edge {
            LeadingEdge::Start => {
                self.logical_cursor.line_end = self.logical_cursor.line_start;
                self.real_cursor.line_end = self.real_cursor.line_start;

                if self.real_cursor.column_start > line_len {
                    self.real_cursor.column_start = line_len;
                    self.logical_cursor.column_start = line_len;
                    self.real_cursor.column_end = self.real_cursor.column_start + 1;
                    self.logical_cursor.column_end = self.logical_cursor.column_start + 1;
                }
                
                self.logical_cursor.column_start = self.logical_cursor.column_start.saturating_sub(1);
                self.logical_cursor.column_end = self.logical_cursor.column_start + 1;
                self.real_cursor.column_start = self.real_cursor.column_start.saturating_sub(1);
                self.real_cursor.column_end = self.real_cursor.column_start + 1;
            }
            LeadingEdge::End => {
                self.logical_cursor.line_start = self.logical_cursor.line_end;
                self.real_cursor.line_start = self.real_cursor.line_end;
                
                self.logical_cursor.column_end = self.logical_cursor.column_end.saturating_sub(1);
                self.real_cursor.column_end = self.logical_cursor.column_end;

                if self.real_cursor.line_end > line_len {
                    self.real_cursor.line_end = line_len;
                    self.logical_cursor.line_end = line_len;
                }
                
                if self.logical_cursor.column_end == 0 {
                    self.logical_cursor.column_start = 0;
                    self.logical_cursor.column_end = 1;
                } else {
                    self.logical_cursor.column_start = self.logical_cursor.column_end - 1;
                    self.real_cursor.column_start = self.real_cursor.column_end - 1;
                }
                
            }
        }
    }

    pub fn move_right(&mut self, line_len: usize) {
        match self.leading_edge {
            LeadingEdge::Start => {
                // Adjust cursor line
                self.logical_cursor.line_end = self.logical_cursor.line_start;
                self.real_cursor.line_end = self.logical_cursor.line_start;
                self.real_cursor.line_start = self.logical_cursor.line_start;

                self.logical_cursor.column_start = self.logical_cursor.column_start.saturating_add(1);
                self.logical_cursor.column_end = self.logical_cursor.column_start + 1;
                self.real_cursor.column_end = self.logical_cursor.column_end;
                self.real_cursor.column_start = self.logical_cursor.column_start;

                // Make real cursor be as long as the line
                if self.logical_cursor.column_start > line_len {
                    self.real_cursor.column_start = line_len;
                    self.real_cursor.column_end = line_len + 1;
                }
            }
            LeadingEdge::End => {
                // Adjust cursor line
                self.logical_cursor.line_start = self.logical_cursor.line_end;
                self.real_cursor.line_start = self.logical_cursor.line_end;
                self.real_cursor.line_end = self.logical_cursor.line_end;

                self.logical_cursor.column_end = self.logical_cursor.column_end.saturating_add(1);
                self.logical_cursor.column_start = self.logical_cursor.column_end - 1;
                self.real_cursor.column_end = self.logical_cursor.column_end;
                self.real_cursor.column_start = self.logical_cursor.column_start;

                // Make real cursor be as long as the line
                if line_len == 0 {
                    self.real_cursor.column_end = line_len + 1;
                    self.real_cursor.column_start = line_len;
                } else if self.logical_cursor.column_end > line_len {
                    self.real_cursor.column_start = line_len - 1;
                    self.real_cursor.column_end = line_len;
                }
            }
        }
    }
}

impl Default for Cursor {
    fn default() -> Self {
        Cursor::new(
            GridCursor::new(0, 0, 0, 1),
            LeadingEdge::Start,
        )
    }
}

impl PartialEq for Cursor {
    fn eq(&self, other: &Cursor) -> bool {
        self.real_cursor == other.real_cursor && self.logical_cursor == other.logical_cursor && self.leading_edge == other.leading_edge
    }
}

#[derive(Copy, Clone, PartialEq, Eq)]
pub struct GridCursor {
    pub line_start: usize,
    pub line_end: usize,
    pub column_start: usize,
    pub column_end: usize,
}

impl GridCursor {
    pub fn new(line_start: usize, line_end: usize, column_start: usize, column_end: usize) -> Self {
        Self {
            line_end,
            line_start,
            column_end,
            column_start
        }
    }
}

impl Default for GridCursor {
    fn default() -> Self {
        GridCursor::new(0, 0, 0, 1)
    }
}
