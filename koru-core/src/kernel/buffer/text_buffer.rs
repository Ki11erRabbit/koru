use std::io::{ErrorKind, SeekFrom};
use std::ops::RangeBounds;
use std::path::{Path, PathBuf};
use crop::{Rope, RopeBuilder, RopeSlice};
use scheme_rs::exceptions::Exception;
use tokio::io::{AsyncSeekExt, AsyncWriteExt};
use crate::kernel::buffer::cursor::{Cursor, CursorDirection};
use crate::kernel::buffer::{Cursors, EditOperation, EditValue, UndoTree};

pub struct TextBuffer {
    buffer: Rope,
    name: String,
    path: Option<PathBuf>,
    undo_tree: UndoTree,
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
            undo_tree: UndoTree::new(),
        }
    }

    pub fn empty<S: Into<String>>(name: S) -> Self {
        TextBuffer {
            buffer: Rope::new(),
            name: name.into(),
            path: None,
            undo_tree: UndoTree::new(),
        }
    }

    pub fn get_name(&self) -> String {
        self.name.clone()
    }
    
    pub fn rename(&mut self, name: String) {
        self.name = name;
    }
    
    pub fn attach_path<P: AsRef<Path>>(&mut self, path: P) {
        self.path = Some(path.as_ref().to_path_buf())
    }
    
    pub fn get_buffer(&self) -> Rope {
        self.buffer.clone()
    }

    pub fn move_cursors(&self, cursors: Vec<Cursor>, direction: CursorDirection) -> Vec<Cursor> {
        let mut new_cursors = Vec::with_capacity(cursors.len());
        for cursor in cursors {
            let cursor = self.move_cursor(cursor, direction);
            new_cursors.push(cursor);
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

    pub fn move_cursor(&self, mut cursor: Cursor, direction: CursorDirection) -> Cursor {
        match direction {
            CursorDirection::Left { wrap } => {
                let at_line_start = cursor.at_line_start();
                if at_line_start && wrap && cursor.line() != 0 {
                    let Some((_, length, end)) = self.buffer.previous_line_information(cursor.line()) else {
                        if cursor.is_main() {
                            return cursor;
                        }
                        return cursor;
                    };
                    cursor.move_up(&self.buffer);
                    cursor.set_column(length);
                } else if !at_line_start {
                    cursor.move_left(self.buffer.line_length(cursor.line()));
                }
                cursor
            }
            CursorDirection::Right {
                wrap,
            } => {
                let at_line_end = cursor.at_line_end(&self.buffer);
                if at_line_end && wrap {
                    cursor.move_down(&self.buffer);
                    let Some((start, _, _)) = self.buffer.previous_line_information(cursor.line()) else {
                        if cursor.is_main() {
                            return cursor;
                        }
                        return cursor;
                    };
                    cursor.set_column(0);
                } else if !at_line_end {
                    cursor.move_right(self.buffer.line_length(cursor.line()));
                }
                cursor
            }
            CursorDirection::Up => {
                cursor.move_up(&self.buffer);
                cursor
            }
            CursorDirection::Down => {
                cursor.move_down(&self.buffer);
                cursor
            }
        }
    }
    
    pub fn place_marks(&self, cursors: Vec<Cursor>) -> Vec<Cursor> {
        let mut output: Vec<Cursor> = Vec::with_capacity(cursors.len());
        for cursor in cursors {
            output.push(self.place_mark(cursor));
        }
        output
    }

    pub fn place_mark(&self, mut cursor: Cursor) -> Cursor {
        cursor.place_mark();
        cursor
    }

    pub fn remove_marks(&self, cursors: Vec<Cursor>) -> Vec<Cursor> {
        let mut output: Vec<Cursor> = Vec::with_capacity(cursors.len());
        for cursor in cursors {
            output.push(self.remove_mark(cursor));
        }
        output
    }

    pub fn remove_mark(&self, mut cursor: Cursor) -> Cursor {
        cursor.remove_mark();
        cursor
    }

    pub fn calculate_byte_offset(&self, line: usize, column: usize) -> usize {
        let mut byte_offset = 0;
        for line in 0..line {
            byte_offset += self.buffer.line_length(line);
        }
        byte_offset += '\n'.len_utf8() * line;
        if self.buffer.line_len() == 0 {
            return byte_offset;
        }

        let line = self.buffer.line(line);
        byte_offset += line.chars()
            .take(column)
            .map(|ch| ch.len_utf8())
            .sum::<usize>();

        byte_offset
    }

    fn insert_text(&mut self, byte_offset: usize, text: &str, cursor_index: usize, cursors: Vec<Cursor>) -> Vec<Cursor> {
        let mut new_cursors = Vec::with_capacity(cursors.len());
        let mut text_after_newline = 0;
        let mut newline_count = 0;

        for ch in text.chars() {
            text_after_newline += 1;
            if ch == '\n' {
                newline_count += 1;
                text_after_newline = 0;
            }
        }
        self.buffer.insert(byte_offset, &text);

        let editor_cursor = cursors[cursor_index];

        for (i, cursor) in cursors.into_iter().enumerate() {
            if i <= cursor_index {
                new_cursors.push(cursor);
            } else if editor_cursor.line() == cursor.line() {
                let mut cursor = cursor;
                for _ in 0..newline_count {
                    cursor = self.move_cursor(cursor, CursorDirection::Down);
                }
                for _ in 0..text_after_newline {
                    cursor = self.move_cursor(cursor, CursorDirection::Right { wrap: false });
                }
                new_cursors.push(cursor);
            } else {
                let mut cursor = cursor;
                for _ in 0..newline_count {
                    cursor = self.move_cursor(cursor, CursorDirection::Down);
                }
                new_cursors.push(cursor);
            }
        }
        new_cursors
    }


    pub async fn insert(&mut self, text: String, cursor_index: usize, cursors: Vec<Cursor>) -> Vec<Cursor>  {
        let byte_offset = self.calculate_byte_offset(cursors[cursor_index].line(), cursors[cursor_index].column());

        let new_cursors = self.insert_text(byte_offset, &text, cursor_index, cursors);

        self.undo_tree.insert(byte_offset, text.clone()).await;
        new_cursors
    }

    pub async fn delete_back(&mut self, cursor_index: usize, cursors: Vec<Cursor>) -> Vec<Cursor> {
        let byte_offset = self.calculate_byte_offset(cursors[cursor_index].line(), cursors[cursor_index].column());
        let line = self.buffer.line(cursors[cursor_index].line());
        let extra_bytes = if cursors[cursor_index].line() != 0 && cursors[cursor_index].at_line_start() {
            '\n'.len_utf8()
        } else {
            0
        };
        let character_offset = byte_offset - line.chars()
            .skip(cursors[cursor_index].column() - 1)
            .take(1)
            .map(|ch| ch.len_utf8())
            .sum::<usize>() - extra_bytes;

        let text = self.buffer.byte_slice(character_offset..byte_offset);
        let text = text.to_string();
        self.buffer.delete(character_offset..byte_offset);

        let new_cursors = self.delete_text(&text, cursor_index, cursors);

        self.undo_tree.delete(character_offset, text).await;
        new_cursors
    }

    pub async fn delete_forward(&mut self, cursor_index: usize, cursors: Vec<Cursor>) -> Vec<Cursor> {
        let byte_offset = self.calculate_byte_offset(cursors[cursor_index].line(), cursors[cursor_index].column());
        let line_no = cursors[cursor_index].line();
        let line = self.buffer.line_slice(line_no..(line_no + 1));
        let character_offset = byte_offset + line.chars()
            .skip(cursors[cursor_index].column())
            .take(1)
            .map(|ch| ch.len_utf8())
            .sum::<usize>();

        let range = byte_offset..character_offset;

        let text = self.buffer.byte_slice(range.clone());
        let text = text.to_string();
        self.buffer.delete(range);

        let new_cursors = self.delete_text(&text, cursor_index, cursors);

        self.undo_tree.delete(byte_offset, text).await;
        new_cursors
    }

    pub async fn delete_region(&mut self, cursor_index: usize, cursors: Vec<Cursor>) -> Vec<Cursor> {
        if !cursors[cursor_index].is_mark_set() {
            return cursors;
        }
        let mark_offset = self.calculate_byte_offset(cursors[cursor_index].mark_line().unwrap(), cursors[cursor_index].mark_column().unwrap());
        let cursor_offset = self.calculate_byte_offset(cursors[cursor_index].line(), cursors[cursor_index].column());

        let (start, range) = if mark_offset < cursor_offset {
            (mark_offset, mark_offset..=cursor_offset)
        } else {
            (cursor_offset, cursor_offset..=(mark_offset - 1))
        };
        let text = self.buffer.byte_slice(range.clone());
        let text = text.to_string();
        self.buffer.delete(range);

        let new_cursors = self.delete_text(&text, cursor_index, cursors);

        self.undo_tree.delete(start - 1, text).await;
        new_cursors
    }

    fn delete_text(&mut self, text: &str, cursor_index: usize, cursors: Vec<Cursor>) -> Vec<Cursor> {
        let mut new_cursors = Vec::with_capacity(cursors.len());
        let mut text_before_newline = 0;
        let mut newline_count = 0;

        for ch in text.chars().rev() {
            text_before_newline += 1;
            if ch == '\n' {
                newline_count += 1;
                text_before_newline = 0;
            }
        }

        let editor_cursor = cursors[cursor_index];

        for (i, cursor) in cursors.into_iter().enumerate() {
            if i <= cursor_index {
                new_cursors.push(editor_cursor);
            } else if editor_cursor.line() == editor_cursor.line() {
                let mut cursor = cursor;
                for _ in 0..newline_count {
                    cursor = self.move_cursor(cursor, CursorDirection::Up);
                }
                for _ in 0..text_before_newline {
                    cursor = self.move_cursor(cursor, CursorDirection::Left { wrap: false });
                }
                new_cursors.push(cursor);
            } else {
                let mut cursor = cursor;
                for _ in 0..newline_count {
                    cursor = self.move_cursor(cursor, CursorDirection::Up);
                }
                new_cursors.push(cursor);
            }
        }
        new_cursors
    }

    pub async fn replace(&mut self, text: String, cursor_index: usize, cursors: Vec<Cursor>) -> Vec<Cursor>  {
        if cursors[cursor_index].is_mark_set() {
            let mark_offset = self.calculate_byte_offset(cursors[cursor_index].mark_line().unwrap(), cursors[cursor_index].mark_column().unwrap());
            let cursor_offset = self.calculate_byte_offset(cursors[cursor_index].line(), cursors[cursor_index].column());

            let (start, range) = if mark_offset < cursor_offset {
                (mark_offset, mark_offset..=cursor_offset)
            } else {
                (cursor_offset, cursor_offset..=(mark_offset - 1))
            };
            let old_text = self.buffer.byte_slice(range);
            let old_text = old_text.to_string();
            self.buffer.delete(mark_offset..cursor_offset);

            let cursors = self.delete_text(&old_text, cursor_index, cursors);
            let cursors = self.insert_text(start, &text, cursor_index, cursors);

            self.undo_tree.replace(start - 1, old_text, text).await;
            cursors
        } else {
            let byte_offset = self.calculate_byte_offset(cursors[cursor_index].line(), cursors[cursor_index].column());
            let line = self.buffer.line(cursors[cursor_index].line());
            let character_offset = byte_offset + line.chars()
                .skip(cursors[cursor_index].column())
                .take(1)
                .map(|ch| ch.len_utf8())
                .sum::<usize>();

            let range = byte_offset..character_offset;

            let old_text = self.buffer.byte_slice(range.clone());
            let old_text = old_text.to_string();

            let cursors = self.delete_text(&old_text, cursor_index, cursors);
            let cursors = self.insert_text(byte_offset, &old_text, cursor_index, cursors);

            self.undo_tree.replace(byte_offset - 1, old_text, text).await;
            cursors
        }
    }

    pub async fn start_transaction(&mut self) {
        self.undo_tree.start_transaction().await;
    }

    pub async fn end_transaction(&mut self) {
        self.undo_tree.end_transaction().await;
    }

    pub fn apply_edit_info(&mut self, edit_info: EditOperation) {
        match edit_info.value {
            EditValue::Insert {
                text
            } => {
                self.buffer.insert(edit_info.byte_offset, text);
            }
            EditValue::Delete {
                count
            } => {
                self.buffer.delete(edit_info.byte_offset..(edit_info.byte_offset + count));
            }
            EditValue::Replace {
                text,
                count
            } => {
                self.buffer.delete(edit_info.byte_offset..(edit_info.byte_offset + count));
                self.buffer.insert(edit_info.byte_offset, text);
            }
            EditValue::Bulk(ops) => {
                for op in ops {
                    self.apply_edit_info(op);
                }
            }
        }
    }

    pub async fn undo(&mut self) {
        let edit_info = self.undo_tree.undo().await;
        let Some(edit_info) = edit_info else {
            return;
        };
        self.apply_edit_info(edit_info);
    }

    pub async fn redo(&mut self) {
        let edit_info = self.undo_tree.redo().await;
        let Some(edit_info) = edit_info else {
            return;
        };
        self.apply_edit_info(edit_info);
    }

    pub async fn save(&mut self) -> Result<(), Exception> {
        let path = self.path.clone()
            .ok_or(Exception::error("Buffer has no associated path"))?;
        self.path = Some(path.clone());
        let mut file = loop {
            let file = tokio::fs::OpenOptions::new()
                .write(true)
                .create(true)
                .truncate(true)
                .open(&path).await;

            match file {
                Ok(file) => {
                    break file;
                }
                Err(err) => {
                    match err.kind() {
                        ErrorKind::Interrupted => {
                            continue;
                        }
                        _ => return Err(Exception::error(err)),
                    }
                }
            }
        };
        let string = self.buffer.to_string();
        file.seek(SeekFrom::Start(0)).await.map_err(|err| Exception::error(err))?;
        file.write_all(string.as_bytes()).await.map_err(|err| Exception::error(err))?;
        file.flush().await.map_err(|err| Exception::error(err))?;
        Ok(())
    }

    pub async fn save_as(&mut self, new_name: &str) -> Result<(), Exception> {
        let path = PathBuf::from(new_name);
        self.path = Some(path);
        self.save().await?;
        Ok(())
    }

    pub fn get_path(&self) -> Option<String> {
        self.path.as_ref().map(|p| p.to_string_lossy().to_string())
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
        Some(self.line_information(line_no.saturating_sub(1)))
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
        if line_no == 0 {
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