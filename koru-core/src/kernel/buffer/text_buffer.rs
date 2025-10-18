use std::path::{Path, PathBuf};
use crate::kernel::cursor::{Cursor, CursorDirection, LeadingEdge};

pub struct TextBuffer {
    buffer: String,
    name: String,
    path: Option<PathBuf>,
}

impl TextBuffer {
    pub fn new<S: Into<String>>(buffer: S, name: S) -> Self {
        TextBuffer {
            buffer: buffer.into(),
            name: name.into(),
            path: None,
        }
    }

    pub fn empty<S: Into<String>>(name: S) -> Self {
        TextBuffer {
            buffer: String::new(),
            name: name.into(),
            path: None,
        }
    }
    
    pub fn rename(&mut self, name: String) {
        self.buffer = name;
    }
    
    pub fn attach_path<P: AsRef<Path>>(&mut self, path: P) {
        self.path = Some(path.as_ref().to_path_buf())
    }
    
    pub fn get_buffer(&self) -> String {
        self.buffer.to_string()
    }

    pub fn move_cursors(&self, cursors: Vec<Cursor>, direction: CursorDirection) -> Vec<Cursor> {
        println!("moving cursors");
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
                let byte_edge = cursor.byte_edge();
                if at_line_start && wrap {
                    cursor.move_logical_up();
                    let Some((_, mut length, end)) = self.buffer.previous_line_information(byte_edge) else {
                        if cursor.is_main() {
                            return Some(cursor);
                        }
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
                } else if !at_line_start {
                    cursor.move_left();
                    let end = byte_edge;
                    let mut end_char_byte = end - 1;
                    while !self.buffer.is_char_boundary(end_char_byte) {
                        end_char_byte -= 1;
                    }
                    let mut start_char_byte = end_char_byte - 1;
                    while !self.buffer.is_char_boundary(start_char_byte) {
                        start_char_byte -= 1;
                    }
                    
                    cursor.byte_cursor.byte_start = start_char_byte;
                    cursor.byte_cursor.byte_end = end_char_byte;
                }
                Some(cursor)
            }
            CursorDirection::Right {
                wrap,
            } => {
                let (at_line_end, passed_edge) = cursor.at_line_end(&self.buffer);
                let byte_edge = cursor.byte_edge();
                if at_line_end && wrap {
                    cursor.move_logical_down(&self.buffer);
                    let Some((start, _, _)) = self.buffer.previous_line_information(byte_edge) else {
                        if cursor.is_main() {
                            return Some(cursor);
                        }
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
                } else if !at_line_end {
                    cursor.move_right();
                    let start = byte_edge;
                    let mut start_char_byte = start;
                    while !self.buffer.is_char_boundary(start_char_byte) {
                        start_char_byte += 1;
                    }
                    let mut end_char_byte = start_char_byte + 1;
                    while !self.buffer.is_char_boundary(end_char_byte) {
                        end_char_byte += 1;
                    }
                    cursor.byte_cursor.byte_end = end_char_byte;
                    cursor.byte_cursor.byte_start = start_char_byte;
                } else if at_line_end && !passed_edge {
                    cursor.move_right();
                }
                Some(cursor)
            }
            CursorDirection::Up => {
                let byte_edge = cursor.byte_edge();
                let Some((start, _, _)) = self.buffer.previous_line_information(byte_edge) else {
                    if cursor.is_main() {
                        return Some(cursor);
                    }
                    return None;
                };
                cursor.move_logical_up();
                match cursor.leading_edge {
                    LeadingEdge::Start => {
                        cursor.logical_cursor.column_end = cursor.logical_cursor.column_start + 1;
                        let (new_start, size) = self.buffer.next_n_chars(start, cursor.logical_cursor.column_start);
                        cursor.byte_cursor.byte_start = new_start;
                        cursor.byte_cursor.byte_end = new_start + size;
                        Some(cursor)
                    }
                    LeadingEdge::End => {
                        cursor.logical_cursor.column_start = cursor.logical_cursor.column_end - 1;
                        let (new_start, size) = self.buffer.next_n_chars(start, cursor.logical_cursor.column_start);
                        cursor.byte_cursor.byte_start = new_start;
                        cursor.byte_cursor.byte_end = new_start + size;
                        Some(cursor)
                    }
                }
            }
            CursorDirection::Down => {
                let byte_edge = cursor.byte_edge();
                let Some((start, _, _)) = self.buffer.next_line_information(byte_edge) else {
                    if cursor.is_main() {
                        return Some(cursor);
                    }
                    return None;
                };
                cursor.move_logical_down(&self.buffer);
                match cursor.leading_edge {
                    LeadingEdge::Start => {
                        cursor.logical_cursor.column_end = cursor.logical_cursor.column_start + 1;
                        let (new_start, size) = self.buffer.next_n_chars(start, cursor.logical_cursor.column_start);
                        cursor.byte_cursor.byte_start = new_start;
                        cursor.byte_cursor.byte_end = new_start + size;
                        Some(cursor)
                    }
                    LeadingEdge::End => {
                        cursor.logical_cursor.column_start = cursor.logical_cursor.column_end - 1;
                        let (new_start, size) = self.buffer.next_n_chars(start, cursor.logical_cursor.column_start);
                        cursor.byte_cursor.byte_start = new_start;
                        cursor.byte_cursor.byte_end = new_start + size;
                        Some(cursor)
                    }
                }
            }
        }
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
    /// Returns a byte position and byte size of that char
    fn next_n_chars(&self, byte_position: usize, n: usize) -> (usize, usize);
    fn previous_n_chars(&self, byte_position: usize, n: usize) -> usize;
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
            if byte_position >= self.len() {
                break;
            }
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
            if byte_position == 0 {
                break;
            }
            if self.as_bytes()[byte_position] == b'\n' {
                prev_line_exists = true;
            }
            byte_position -= 1;
        }
        prev_line_exists
    }

    fn line_start(&self, byte_position: usize) -> usize {
        let byte_position = if self.as_bytes()[byte_position] == b'\n' {
            byte_position - 1
        } else {
            byte_position
        };
        
        let start = {
            let mut byte_position = byte_position;
            loop {
                if byte_position == 0 {
                    break byte_position;
                }
                if self.as_bytes()[byte_position] == b'\n' {
                    break byte_position + 1;
                }
                byte_position -= 1;
            }
        };
        start
    }


    fn line_end(&self, byte_position: usize) -> usize {
        if self.as_bytes()[byte_position] == b'\n' {
            return byte_position;
        }
        let end = {
            let mut byte_position = byte_position;
            loop {
                if byte_position >= self.len() {
                    break byte_position;
                }
                if self.as_bytes()[byte_position] == b'\n' {
                    break byte_position - 1;
                }
                byte_position += 1;
            }
        };
        end
    }

    fn line_information(&self, byte_position: usize) -> (usize, usize, usize) {
        if self.as_bytes()[byte_position] == b'\n' {
            return (byte_position - 1, 0, byte_position);
        }
        
        let start = {
            let mut byte_position = byte_position;
            loop {
                if byte_position == 0 {
                    break byte_position;
                }
                if self.as_bytes()[byte_position] == b'\n' {
                    break byte_position + 1;
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
                    break byte_position - 1;
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
                // Ensures that we are at least before the trailing newline
                break byte_position - 1;
            }
            byte_position -= 1;
        };

        Some(self.line_information(byte_position))
    }

    fn next_line_information(&self, mut byte_position: usize) -> Option<(usize, usize, usize)> {
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
            byte_position += 1;
        };
        Some(self.line_information(byte_position))
    }

    fn next_n_chars(&self, byte_position: usize, mut n: usize) -> (usize, usize) {
        let mut byte_position = byte_position;
        while n != 0 && byte_position < self.len() {
            if self.is_char_boundary(byte_position) {
                n -= 1
            }
            byte_position += 1;
        }
        let mut size = 1;
        while !self.is_char_boundary(byte_position + size){
            size += 1;
        }

        (byte_position, size)
    }

    fn previous_n_chars(&self, byte_position: usize, mut n: usize) -> usize {
        let mut byte_position = byte_position - 1;
        while n != 0 && byte_position != 0 {
            if self.is_char_boundary(byte_position) {
                n -= 1
            }
            byte_position -= 1;
        }
        byte_position
    }
}