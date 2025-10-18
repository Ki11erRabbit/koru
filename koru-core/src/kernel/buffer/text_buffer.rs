use crate::kernel::cursor::{Cursor, CursorDirection, LeadingEdge};

pub struct TextBuffer {
    buffer: String,
    name: String,
}

impl TextBuffer {
    pub fn new<S: Into<String>>(buffer: S, name: S) -> Self {
        TextBuffer {
            buffer: buffer.into(),
            name: name.into(),
        }
    }

    pub fn empty<S: Into<String>>(name: S) -> Self {
        TextBuffer {
            buffer: String::new(),
            name: name.into(),
        }
    }

    pub fn move_cursors(&self, cursors: Vec<Cursor>, direction: CursorDirection) -> Vec<Cursor> {
        let mut new_cursors = Vec::with_capacity(cursors.len());
        for cursor in cursors {
            if let Some(cursor) = self.move_cursor(cursor, direction) {
                new_cursors.push(cursor);
            }
        }
        new_cursors
    }

    pub fn move_cursor(&self, mut cursor: Cursor, direction: CursorDirection) -> Option<Cursor> {
        match direction {
            CursorDirection::Left { wrap } => {
                let at_line_start = cursor.at_line_start();
                let byte_edge = cursor.byte_edge();
                if at_line_start && wrap {
                    cursor.move_logical_up();
                    let Some((_, mut length, end)) = self.buffer.previous_line_information(byte_edge) else {
                        return None;
                    };
                    if length == 0 {
                        length += 1;
                    }
                    cursor.logical_cursor.column_end = length;
                    cursor.logical_cursor.column_start = length - 1;
                    cursor.byte_cursor.byte_end = end;
                    let mut start_char_byte = end - 1;
                    while !self.buffer.is_char_boundary(start_char_byte) {
                        start_char_byte -= 1;
                    }
                    cursor.byte_cursor.byte_start = start_char_byte;
                } else  {
                    
                }
            }
            CursorDirection::Right {
                wrap,
            } => {
                let at_line_start = cursor.at_line_start();
                let at_line_end = cursor.at_line_end(self);
                if at_line_end && wrap {
                    cursor.move_logical_down(&self.buffer);
                    let Some((start, _, _)) = self.buffer.previous_line_information(cursor.byte_cursor.byte_start) else {
                        return None;
                    };
                    cursor.logical_cursor.column_end = 1;
                    cursor.logical_cursor.column_start = 0;
                    cursor.byte_cursor.byte_start = start;
                    let mut end_char_byte = start + 1;
                    while !self.buffer.is_char_boundary(end_char_byte) {
                        end_char_byte += 1;
                    }
                    cursor.byte_cursor.byte_end = end_char_byte;
                } else if at_line_end {
                    
                }
            }
        }
        None
    }

}

pub trait TextBufferImpl {
    fn line_length(&self, byte_position: usize) -> usize;
    fn is_there_next_line(&self, byte_position: usize) -> bool;
    fn is_there_prev_line(&self, byte_position: usize) -> bool;
    /// Returns a byte position for the start of a line
    fn line_start(&self, byte_position: usize) -> usize;
    /// Returns a byte position
    fn line_end(&self, byte_position: usize) -> usize;
    /// Should return information about the current line.
    /// `Returns`: byte start, line_length in chars, byte_end
    fn line_information(&self, byte_position: usize) -> (usize, usize, usize);
    fn previous_line_information(&self, byte_position: usize) -> Option<(usize, usize, usize)>;
    fn next_line_information(&self, byte_position: usize) -> Option<(usize, usize, usize)>;
}

impl TextBufferImpl for String {
    fn line_length(&self, byte_position: usize) -> usize {
        let start = self.line_start(byte_position);
        let end = self.line_end(byte_position);

        let string: &str = &self[start..end];

        string.chars().count()
    }

    fn is_there_next_line(&self, mut byte_position: usize) -> bool {
        let mut next_line_exists = false;
        while self.len() > byte_position {
            if self.as_bytes()[byte_position] == b'\n' {
                next_line_exists = true;
            }
            byte_position += 1;
        }
        next_line_exists
    }

    fn is_there_prev_line(&self, mut byte_position: usize) -> bool {
        let mut prev_line_exists = false;
        while self.len() > byte_position {
            if self.as_bytes()[byte_position] == b'\n' {
                prev_line_exists = true;
            }
            byte_position -= 1;
        }
        prev_line_exists
    }

    fn line_start(&self, byte_position: usize) -> usize {
        let start = {
            let mut byte_position = byte_position;
            loop {
                if byte_position == 0 {
                    break byte_position;
                }
                if self.as_bytes()[byte_position] == b'\n' {
                    break byte_position;
                }
                byte_position -= 1;
            }
        };
        start
    }


    fn line_end(&self, byte_position: usize) -> usize {
        let end = {
            let mut byte_position = byte_position;
            loop {
                if self.len() >= byte_position {
                    break byte_position;
                }
                if self.as_bytes()[byte_position] == b'\n' {
                    break byte_position;
                }
                byte_position += 1;
            }
        };
        end
    }

    fn line_information(&self, byte_position: usize) -> (usize, usize, usize) {
        let start = {
            let mut byte_position = byte_position;
            loop {
                if byte_position == 0 {
                    break byte_position;
                }
                if self.as_bytes()[byte_position] == b'\n' {
                    break byte_position;
                }
                byte_position -= 1;
            }
        };
        let end = {
            let mut byte_position = byte_position;
            loop {
                if self.len() >= byte_position {
                    break byte_position;
                }
                if self.as_bytes()[byte_position] == b'\n' {
                    break byte_position;
                }
                byte_position += 1;
            }
        };

        let string: &str = &self[start..end];

        (start, string.chars().count(), end)
    }

    fn previous_line_information(&self, mut byte_position: usize) -> Option<(usize, usize, usize)> {
        if !self.is_there_prev_line(byte_position) {
            return None;
        }
        let byte_position = loop {
            if byte_position == 0 {
                break byte_position;
            }
            if self.as_bytes()[byte_position] == b'\n' {
                // Ensures that we are at least before the newline
                break byte_position.saturating_sub(1);
            }
            byte_position -= 1;
        };

        Some(self.line_information(byte_position))
    }

    fn next_line_information(&self, byte_position: usize) -> Option<(usize, usize, usize)> {
        if !self.is_there_next_line(byte_position) {
            return None;
        }
        let byte_position = loop {
            if byte_position >= self.len() {
                break byte_position;
            }
            if self.as_bytes()[byte_position] == b'\n' {
                // Ensures that we are at least after the newline
                break byte_position + 1;
            }
        };
        Some(self.line_information(byte_position))
    }
}