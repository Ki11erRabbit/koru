use std::path::{Path, PathBuf};
use crop::{Rope, RopeBuilder};
use crate::kernel::buffer::cursor::{Cursor, CursorDirection, LeadingEdge};

pub struct TextBuffer {
    buffer: Rope,
    name: String,
    path: Option<PathBuf>,
}

impl TextBuffer {
    pub fn new<S: Into<String>>(buffer: S, name: S) -> Self {
        let text = buffer.into();
        let mut builder = RopeBuilder::new();
        builder.append(text);


        TextBuffer {
            buffer: builder.build(),
            name: name.into(),
            path: None,
        }
    }

    pub fn empty<S: Into<String>>(name: S) -> Self {
        TextBuffer {
            buffer: Rope::new(),
            name: name.into(),
            path: None,
        }
    }
    
    pub fn rename(&mut self, name: String) {
        self.name = name;
    }
    
    pub fn attach_path<P: AsRef<Path>>(&mut self, path: P) {
        self.path = Some(path.as_ref().to_path_buf())
    }
    
    pub fn get_buffer(&self) -> String {
        self.buffer.to_string()
    }

    pub fn move_cursors(&self, cursors: Vec<Cursor>, direction: CursorDirection) -> Vec<Cursor> {
        let mut new_cursors = Vec::with_capacity(cursors.len());
        for cursor in cursors {
            if let Some(cursor) = self.move_cursor(cursor, direction) {
                new_cursors.push(cursor);
            }
        }
        let mut index = 1;
        // Remove a cursor if it matches the main cursor or a cursor next to it is the same
        while index < new_cursors.len() {
            if new_cursors[index - 1].is_main() && new_cursors[index - 1] == new_cursors[index] {
                new_cursors.remove(index);
                continue;
            }
            if new_cursors[index].is_main() && new_cursors[index - 1] == new_cursors[index] {
                new_cursors.remove(index - 1);
                index -= 1;
                continue;
            }
            if new_cursors[index] == new_cursors[index - 1] {
                new_cursors.remove(index);
                continue;
            }
            index += 1;
        }
        new_cursors
    }

    pub fn move_cursor(&self, mut cursor: Cursor, direction: CursorDirection) -> Option<Cursor> {
        match direction {
            CursorDirection::Left { wrap } => {
                let at_line_start = cursor.at_line_start();
                if at_line_start && wrap {
                    cursor.move_up(&self.buffer);
                    let Some((_, mut length, end)) = self.buffer.previous_line_information(cursor.line()) else {
                        if cursor.is_main() {
                            return Some(cursor);
                        }
                        return None;
                    };
                    if length == 0 {
                        length += 1;
                    }
                    cursor.set_column_end(length);
                    cursor.set_column_start(length - 1);
                } else if !at_line_start {
                    cursor.move_left(self.buffer.line_length(cursor.line()));
                }
                Some(cursor)
            }
            CursorDirection::Right {
                wrap,
            } => {
                let at_line_end = cursor.at_line_end(&self.buffer);
                if at_line_end && wrap {
                    cursor.move_down(&self.buffer);
                    let Some((start, _, _)) = self.buffer.previous_line_information(cursor.line()) else {
                        if cursor.is_main() {
                            return Some(cursor);
                        }
                        return None;
                    };
                    cursor.set_column_start(0);
                    cursor.set_column_end(1);
                } else if !at_line_end {
                    cursor.move_right(self.buffer.line_length(cursor.line()));
                }
                Some(cursor)
            }
            CursorDirection::Up => {
                cursor.move_up(&self.buffer);
                Some(cursor)
            }
            CursorDirection::Down => {
                cursor.move_down(&self.buffer);
                Some(cursor)
            }
        }
    }

}

pub trait TextBufferImpl {
    fn line_length(&self, line_no: usize) -> usize;
    fn is_there_next_line(&self, line_no: usize) -> bool;
    fn is_there_prev_line(&self, line_no: usize) -> bool;
    /// Returns a byte position for the start of a line
    fn line_start(&self, line_no: usize) -> usize;
    /// Returns a byte position
    fn line_end(&self, line_no: usize) -> usize;
    /// Should return information about the current line.
    /// `Returns`: byte start, line_length in chars, byte_end
    fn line_information(&self, line_no: usize) -> (usize, usize, usize);
    fn previous_line_information(&self, line_no: usize) -> Option<(usize, usize, usize)> {
        if !self.is_there_prev_line(line_no) {
            return None;
        }
        Some(self.line_information(line_no - 1))
    }
    fn next_line_information(&self, line_no: usize) -> Option<(usize, usize, usize)> {
        if !self.is_there_next_line(line_no) {
            return None;
        }
        Some(self.line_information(line_no + 1))
    }
    /// Returns a byte position and byte size of that char
    fn next_n_chars(&self, line_no: usize, n: usize) -> (usize, usize);
}

impl TextBufferImpl for Rope {
    fn line_length(&self, line_no: usize) -> usize {
        self.line(line_no).byte_len()
    }

    fn is_there_next_line(&self, line_no: usize) -> bool {
        if self.line_len() > line_no + 1 {
            true
        } else {
            false
        }
    }

    fn is_there_prev_line(&self, line_no: usize) -> bool {
        if line_no != 0 {
            false
        } else {
            true
        }
    }

    fn line_start(&self, line_no: usize) -> usize {
        self.byte_of_line(line_no)
    }

    fn line_end(&self, line_no: usize) -> usize {
        self.byte_of_line(line_no) + self.line(line_no).byte_len()
    }

    fn line_information(&self, line_no: usize) -> (usize, usize, usize) {
        let start = self.line_start(line_no);
        let end = self.line_end(line_no);
        let len = self.line(line_no).chars().count();

        (start, len, end)
    }

    fn next_n_chars(&self, line_no: usize, n: usize) -> (usize, usize) {
        let line = self.line(line_no);
        let mut pos = self.byte_of_line(line_no);
        let mut size = 0;
        for (ch, _) in line.chars().zip(0..n) {
            pos += ch.len_utf8();
            size = ch.len_utf8();
        }
        (pos, size)
    }
}