use std::borrow::Cow;
use std::collections::HashMap;
use std::fmt::Display;
use std::sync::{Arc};
use bitflags::bitflags;
use crop::Rope;
use scheme_rs::exceptions::Exception;
use scheme_rs::gc::{Gc, Trace};
use scheme_rs::num::SimpleNumber;
use scheme_rs::records::{rtd, Record, RecordTypeDescriptor, SchemeCompatible};
use scheme_rs::registry::bridge;
use scheme_rs::value::Value;
use unicode_normalization::char::is_combining_mark;
use crate::kernel::buffer::Cursor;

#[derive(Clone, Debug, Eq, PartialEq, Trace)]
pub struct TextChunk {
    #[trace(skip)]
    rope: Rope,
    start: usize,
    end: usize,
}
impl TextChunk {
    pub fn new(rope: Rope, start: usize, end: usize) -> Self {
        Self { rope, start, end }
    }

    pub fn chars(&self) -> impl Iterator<Item = char> + '_ {
        self.rope.byte_slice(self.start..self.end).chars()
    }

    pub fn graphemes(&self) -> impl Iterator<Item = Cow<str>> {
        self.rope.byte_slice(self.start..self.end).graphemes()
    }

    pub fn start(&self) -> usize {
        self.start
    }
    pub fn end(&self) -> usize {
        self.end
    }
}

impl SchemeCompatible for TextChunk {
    fn rtd() -> Arc<RecordTypeDescriptor>
    where
        Self: Sized
    {
        rtd!(name: "&TextChunk", sealed: true)
    }
}

impl Display for TextChunk {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.rope.byte_slice(self.start..self.end).to_string())
    }
}

impl From<String> for TextChunk {
    fn from(text: String) -> Self {
        let len = text.len();
        TextChunk::new(Rope::from(text), 0, len)
    }
}


bitflags! {
    #[derive(Debug, Copy, Clone, Eq, PartialEq, Hash)]
    pub struct TextAttribute: u8 {
        const Italic = 0b0000_0001;
        const Bold = 0b0000_0010;
        const Strikethrough = 0b0000_0100;
        const Underline = 0b0000_1000;
    }
}


#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash, Trace)]
pub enum ColorType {
    Base,
    SecondaryBase,
    TertiaryBase,
    Surface0,
    Surface1,
    Surface2,
    Overlay0,
    Overlay1,
    Overlay2,
    Text,
    Subtext0,
    Subtext1,
    Accent,
    Link,
    Success,
    Warning,
    Error,
    Tags,
    Selection,
    Cursor,
    SecondaryCursor,
    Type,
    Interface,
    Function,
    Method,
    Macro,
    Keyword,
    Comment,
    String,
    Literal,
    Operator,
    Pink,
    Red,
    Lime,
    Green,
    LightYellow,
    Yellow,
    Orange,
    Brown,
    LightBlue,
    Blue,
    LightMagenta,
    Magenta,
    LightPurple,
    Purple,
    LightCyan,
    Cyan,
    White,
    LightGray,
    Gray,
    Black,
}

impl TryFrom<&str> for ColorType {
    type Error = String;

    fn try_from(value: &str) -> Result<Self, <ColorType as TryFrom<&str>>::Error> {
        let value = match value {
            "Base" => ColorType::Base,
            "SecondaryBase" => ColorType::SecondaryBase,
            "TertiaryBase" => ColorType::TertiaryBase,
            "Surface0" => ColorType::Surface0,
            "Surface1" => ColorType::Surface1,
            "Surface2" => ColorType::Surface2,
            "Overlay0" => ColorType::Overlay0,
            "Overlay1" => ColorType::Overlay1,
            "Overlay2" => ColorType::Overlay2,
            "Text" => ColorType::Text,
            "Subtext0" => ColorType::Subtext0,
            "Subtext1" => ColorType::Subtext1,
            "Accent" => ColorType::Accent,
            "Link" => ColorType::Link,
            "Success" => ColorType::Success,
            "Warning" => ColorType::Warning,
            "Error" => ColorType::Error,
            "Tags" => ColorType::Tags,
            "Selection" => ColorType::Selection,
            "Cursor" => ColorType::Cursor,
            "SecondaryCursor" => ColorType::SecondaryCursor,
            "Pink" => ColorType::Pink,
            "Red" => ColorType::Red,
            "Lime" => ColorType::Lime,
            "Green" => ColorType::Green,
            "LightYellow" => ColorType::LightYellow,
            "Yellow" => ColorType::Yellow,
            "Orange" => ColorType::Orange,
            "Brown" => ColorType::Brown,
            "LightBlue" => ColorType::LightBlue,
            "Blue" => ColorType::Blue,
            "LightMagenta" => ColorType::LightMagenta,
            "Magenta" => ColorType::Magenta,
            "LightPurple" => ColorType::LightPurple,
            "Purple" => ColorType::Purple,
            "LightCyan" => ColorType::LightCyan,
            "Cyan" => ColorType::Cyan,
            "White" => ColorType::White,
            "LightGray" => ColorType::LightGray,
            "Gray" => ColorType::Gray,
            "Black" => ColorType::Black,
            x => return Err(format!("invalid color type '{}'", x)),
        };
        Ok(value)
    }
}

impl SchemeCompatible for ColorType {
    fn rtd() -> Arc<RecordTypeDescriptor>
    where
        Self: Sized
    {
        rtd!(name: "&ColorType", sealed: true)
    }
}


#[derive(Debug, Clone, Eq, PartialEq, Trace)]
pub enum StyledText {

    None {
        text: TextChunk,
    },
    Style {
        fg_color: ColorType,
        bg_color: ColorType,
        #[trace(skip)]
        attribute: TextAttribute,
        text: TextChunk,
    }
}


impl SchemeCompatible for StyledText {
    fn rtd() -> Arc<RecordTypeDescriptor>
    where
        Self: Sized
    {
        rtd!(name: "&StyledText", sealed: true)
    }
}

#[bridge(name = "styled-text-create", lib = "(styled-text)")]
pub fn styled_text_create(args: &[Value]) -> Result<Vec<Value>, Exception> {
    let Some((text, rest)) = args.split_first() else {
        return Err(Exception::wrong_num_of_args(1, args.len()));
    };
    let text: String = text.clone().try_into()?;
    if let Some((fg_color, rest)) = rest.split_first() {
        let fg_color: String = fg_color.clone().try_into()?;
        let Some((bg_color, rest)) = rest.split_first() else {
            return Err(Exception::wrong_num_of_args(3, args.len()));
        };
        let bg_color: String = bg_color.clone().try_into()?;
        let fg_color = match fg_color.as_str().try_into()  {
            Ok(color) => color,
            Err(msg) => {
                return Err(Exception::error(msg));
            }
        };
        let bg_color = match bg_color.as_str().try_into()  {
            Ok(color) => color,
            Err(msg) => {
                return Err(Exception::error(msg));
            }
        };

        let mut attributes = TextAttribute::empty();

        for attr in rest {
            let attr: String = attr.clone().try_into()?;
            match attr.as_str() {
                "italic" => attributes |= TextAttribute::Italic,
                "bold" => attributes |= TextAttribute::Bold,
                "strikethrough" => attributes |= TextAttribute::Strikethrough,
                "underline" => attributes |= TextAttribute::Underline,
                _ => {
                    return Err(Exception::error(String::from("attribute is not one of 'italic, 'bold, 'strikethrough, or 'underline")));
                }
            }
        }

        let text_len = text.len();
        let text = TextChunk::new(Rope::from(text), 0, text_len);

        let text = StyledText::Style {
            fg_color,
            bg_color,
            attribute: attributes,
            text,
        };
        Ok(vec![Value::from(Record::from_rust_type(text))])

    } else {
        let text_len = text.len();
        let text = TextChunk::new(Rope::from(text), 0, text_len);
        let text = StyledText::None { text };
        Ok(vec![Value::from(Record::from_rust_type(text))])
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
struct Position {
    line: usize,
    column: usize,
}

impl Position {
    fn new(line: usize, column: usize) -> Self {
        Self { line, column }
    }
}

#[derive(Debug, Clone, Copy)]
enum SelectionType {
    Point,
    Line,
    Box,
    File,
}

#[derive(Debug, Clone, Copy)]
struct Selection {
    start: Position,
    end: Position,
    selection_type: SelectionType,
}

impl Selection {
    fn contains(&self, pos: Position) -> bool {
        match self.selection_type {
            SelectionType::Point => {
                self.is_point_in_range(pos)
            }
            SelectionType::Line => {
                self.is_line_in_range(pos.line)
            }
            SelectionType::Box => {
                self.is_box_in_range(pos)
            }
            SelectionType::File => true,
        }
    }

    fn is_point_in_range(&self, pos: Position) -> bool {
        let (start, end) = if self.start.line < self.end.line
            || (self.start.line == self.end.line && self.start.column <= self.end.column) {
            (self.start, self.end)
        } else {
            (self.end, self.start)
        };

        if pos.line < start.line || pos.line > end.line {
            return false;
        }

        if pos.line == start.line && pos.line == end.line {
            // Same line: inclusive on both ends
            pos.column >= start.column && pos.column <= end.column
        } else if pos.line == start.line {
            // Start line: from start column to end of line
            pos.column >= start.column
        } else if pos.line == end.line {
            // End line: from start of line to end column (inclusive)
            pos.column <= end.column
        } else {
            // Middle lines: entire line is selected
            true
        }
    }

    fn is_line_in_range(&self, line: usize) -> bool {
        let (start_line, end_line) = if self.start.line <= self.end.line {
            (self.start.line, self.end.line)
        } else {
            (self.end.line, self.start.line)
        };

        line >= start_line && line <= end_line
    }

    fn is_box_in_range(&self, pos: Position) -> bool {
        let (min_line, max_line) = if self.start.line <= self.end.line {
            (self.start.line, self.end.line)
        } else {
            (self.end.line, self.start.line)
        };

        let (min_col, max_col) = if self.start.column <= self.end.column {
            (self.start.column, self.end.column)
        } else {
            (self.end.column, self.start.column)
        };

        pos.line >= min_line && pos.line <= max_line
            && pos.column >= min_col && pos.column <= max_col
    }
}

#[derive(Debug, Clone, Eq, PartialEq, Trace)]
pub struct StyledFile {
    #[trace(skip)]
    lines: Vec<Vec<StyledText>>,
}

impl StyledFile {
    pub fn new() -> Self {
        Self {
            lines: Vec::new(),
        }
    }

    pub fn lines(&self) -> &Vec<Vec<StyledText>> {
        &self.lines
    }

    pub fn push_line(&mut self, line: Vec<StyledText>) {
        self.lines.push(line);
    }

    pub fn line_count(&self) -> usize {
        self.lines.len()
    }

    pub fn prepend_segment(&mut self, line: usize, text: StyledText) {
        while self.lines.len() <= line {
            self.lines.push(Vec::new());
        }
        self.lines[line].insert(0, text);
    }

    pub fn append_segment(&mut self, line: usize, text: StyledText) {
        while self.lines.len() <= line {
            self.lines.push(Vec::new());
        }
        self.lines[line].push(text);
    }
}

impl StyledFile {
    /// Build a map of positions to their selection/cursor states
    fn build_selection_map(cursors: &[Cursor]) -> HashMap<Position, Vec<(usize, bool, bool)>> {
        let mut map: HashMap<Position, Vec<(usize, bool, bool)>> = HashMap::new();

        for (idx, cursor) in cursors.iter().enumerate() {
            let cursor_pos = Position::new(cursor.line(), cursor.column());

            // Mark cursor position
            map.entry(cursor_pos)
                .or_insert_with(Vec::new)
                .push((idx, true, false));

            // Mark selection if present
            if let Some(mark_line) = cursor.mark_line() {
                if let Some(mark_col) = cursor.mark_column() {
                    let mark_pos = Position::new(mark_line, mark_col);
                    map.entry(mark_pos)
                        .or_insert_with(Vec::new)
                        .push((idx, false, true));
                }
            }
        }

        map
    }

    fn process_segment_with_selections(
        selection_map: &HashMap<Position, Vec<(usize, bool, bool)>>,
        cursors: &[Cursor],
        line_index: usize,
        current_line: &mut Vec<StyledText>,
        column_index: &mut usize,
        text: TextChunk,
        fg_color: ColorType,
        bg_color: ColorType,
        attribute: TextAttribute,
    ) {
        // Pre-compute all selections for this line to avoid repeated calculations
        let mut active_selections: Vec<Selection> = Vec::new();
        for cursor in cursors {
            if !cursor.is_mark_set() {
                continue;
            }

            let cursor_pos = Position::new(cursor.line(), cursor.column());
            let mark_pos = Position::new(
                cursor.mark_line().unwrap(),
                cursor.mark_column().unwrap()
            );

            let selection_type = match cursor.mark_state {
                crate::kernel::buffer::CursorMark::Point => SelectionType::Point,
                crate::kernel::buffer::CursorMark::Line => SelectionType::Line,
                crate::kernel::buffer::CursorMark::Box => SelectionType::Box,
                crate::kernel::buffer::CursorMark::File => SelectionType::File,
                crate::kernel::buffer::CursorMark::None => continue,
            };

            active_selections.push(Selection {
                start: cursor_pos,
                end: mark_pos,
                selection_type,
            });
        }

        // Check if this is an empty line (just a newline character)
        let is_empty_line = text.chars().next() == Some('\n') &&
            text.chars().count() == 1;

        let mut start = text.start();
        let mut current_pos_bytes = start;
        let mut prev_column = *column_index;

        // Handle empty line with cursor - prepend a space before the newline
        if is_empty_line {
            let pos = Position::new(line_index, *column_index);
            let cursor_at_pos = selection_map.get(&pos)
                .and_then(|info_list| {
                    info_list.iter()
                        .find(|&&(_idx, is_cursor, _is_mark)| is_cursor)
                        .map(|&(idx, _, _)| idx)
                });

            if let Some(cursor_idx) = cursor_at_pos {
                // Render cursor as a space on empty line
                let cursor = &cursors[cursor_idx];
                let cursor_color = if cursor.is_main() {
                    ColorType::Cursor
                } else {
                    ColorType::SecondaryCursor
                };

                current_line.push(StyledText::Style {
                    fg_color,
                    bg_color: cursor_color,
                    attribute: TextAttribute::empty(),
                    text: TextChunk::from(String::from(' ')),
                });
            } else {
                // Check if line is selected
                let is_selected = active_selections.iter()
                    .any(|sel| sel.contains(pos));

                let line_bg = if is_selected {
                    ColorType::Selection
                } else {
                    bg_color
                };

                // Render empty line as a space to maintain visibility
                current_line.push(StyledText::Style {
                    fg_color,
                    bg_color: line_bg,
                    attribute,
                    text: TextChunk::from(String::from(' ')),
                });
            }

            // Increment column for the space we just added
            *column_index += 1;
            prev_column = *column_index;
        }


        for grapheme in text.graphemes() {
            let grapheme_len = grapheme.len();
            let next_pos_bytes = current_pos_bytes + grapheme_len;
            let current_column = *column_index;
            let is_newline = grapheme.contains('\n');

            // Check if any cursor is at this position (BEFORE incrementing column)
            let pos = Position::new(line_index, current_column);
            let cursor_at_pos = selection_map.get(&pos)
                .and_then(|info_list| {
                    info_list.iter()
                        .find(|&&(_idx, is_cursor, _is_mark)| is_cursor)
                        .map(|&(idx, _, _)| idx)
                });

            // Check if this position is selected
            let is_selected = active_selections.iter()
                .any(|sel| sel.contains(pos));

            // Determine if we need to flush accumulated text
            if let Some(cursor_idx) = cursor_at_pos {
                // Flush text before cursor if any
                if start < current_pos_bytes {
                    let prev_pos = Position::new(line_index, prev_column);
                    let prev_selected = active_selections.iter()
                        .any(|sel| sel.contains(prev_pos));

                    let chunk_bg = if prev_selected {
                        ColorType::Selection
                    } else {
                        bg_color
                    };

                    current_line.push(StyledText::Style {
                        fg_color,
                        bg_color: chunk_bg,
                        attribute,
                        text: TextChunk::new(text.rope.clone(), start, current_pos_bytes),
                    });
                }

                // Render cursor
                let cursor = &cursors[cursor_idx];
                let cursor_color = if cursor.is_main() {
                    ColorType::Cursor
                } else {
                    ColorType::SecondaryCursor
                };

                let cursor_text = if is_newline {
                    // Cursor at end of line - show as space, but still include the newline after
                    current_line.push(StyledText::Style {
                        fg_color,
                        bg_color: cursor_color,
                        attribute: TextAttribute::empty(),
                        text: TextChunk::from(String::from(' ')),
                    });

                    // Now add the actual newline
                    start = current_pos_bytes;
                    current_pos_bytes = next_pos_bytes;
                    *column_index += 1;
                    prev_column = current_column + 1;
                    continue;
                } else {
                    // Cursor on grapheme
                    TextChunk::new(text.rope.clone(), current_pos_bytes, next_pos_bytes)
                };

                current_line.push(StyledText::Style {
                    fg_color,
                    bg_color: cursor_color,
                    attribute: TextAttribute::empty(),
                    text: cursor_text,
                });

                start = next_pos_bytes;
                prev_column = current_column + 1;
            } else if start < current_pos_bytes {
                // Check if background changed (selected to unselected or vice versa)
                let prev_pos = Position::new(line_index, prev_column);
                let prev_selected = active_selections.iter()
                    .any(|sel| sel.contains(prev_pos));

                if prev_selected != is_selected {
                    // Flush accumulated text with previous background
                    let chunk_bg = if prev_selected {
                        ColorType::Selection
                    } else {
                        bg_color
                    };

                    current_line.push(StyledText::Style {
                        fg_color,
                        bg_color: chunk_bg,
                        attribute,
                        text: TextChunk::new(text.rope.clone(), start, current_pos_bytes),
                    });

                    start = current_pos_bytes;
                    prev_column = current_column;
                }
            }

            current_pos_bytes = next_pos_bytes;
            *column_index += 1;
        }

        // Push any remaining text
        if start < current_pos_bytes {
            let final_pos = Position::new(line_index, prev_column);
            let final_selected = active_selections.iter()
                .any(|sel| sel.contains(final_pos));

            let final_bg = if final_selected {
                ColorType::Selection
            } else {
                bg_color
            };

            current_line.push(StyledText::Style {
                fg_color,
                bg_color: final_bg,
                attribute,
                text: TextChunk::new(text.rope.clone(), start, current_pos_bytes),
            });
        }

        // Check if this line has a line selection but no newline at the end
        // This handles the edge case where the last line doesn't end with \n
        let line_has_no_newline = !text.chars().any(|ch| ch == '\n');
        if line_has_no_newline {
            let end_pos = Position::new(line_index, *column_index);
            let has_line_selection = active_selections.iter()
                .any(|sel| matches!(sel.selection_type, SelectionType::Line) && sel.contains(end_pos));

            if has_line_selection {
                // Add a visual space to show the line is selected
                current_line.push(StyledText::Style {
                    fg_color,
                    bg_color: ColorType::Selection,
                    attribute,
                    text: TextChunk::from(String::from(' ')),
                });
            }
        }
    }

    /// Place cursors and their selections into the styled text.
    /// Cursors can be in arbitrary order.
    pub fn place_cursors(self, cursors: &[Cursor]) -> Self {
        let selection_map = Self::build_selection_map(cursors);

        let mut lines = Vec::new();

        for (line_index, line) in self.lines.into_iter().enumerate() {
            let mut current_line = Vec::new();
            let mut column_index = 0;

            for segment in line {
                let (fg_color, bg_color, attribute, text) = match segment {
                    StyledText::None { text } => {
                        (ColorType::Text, ColorType::Base, TextAttribute::empty(), text)
                    }
                    StyledText::Style { fg_color, bg_color, attribute, text } => {
                        (fg_color, bg_color, attribute, text)
                    }
                };

                Self::process_segment_with_selections(
                    &selection_map,
                    cursors,
                    line_index,
                    &mut current_line,
                    &mut column_index,
                    text,
                    fg_color,
                    bg_color,
                    attribute,
                );
            }

            lines.push(current_line);
        }

        Self { lines }
    }
}



impl Default for StyledFile {
    fn default() -> Self {
        let rope = Rope::new();
        StyledFile::from(rope)
    }
}

impl From<Rope> for StyledFile {
    fn from(text: Rope) -> Self {
        let mut start = 0;
        let mut end = 0;
        let mut lines = Vec::new();
        for ch in text.chars() {
            end += ch.len_utf8();
            if ch == '\n' {
                lines.push(vec![StyledText::None {
                    text: TextChunk::new(text.clone(), start, end)
                }]);
                start = end;
            }
        }
        lines.push(vec![StyledText::None {
            text: TextChunk::new(text.clone(), start, end)
        }]);

        Self {
            lines
        }
    }
}

impl From<String> for StyledFile {
    fn from(text: String) -> Self {
        let rope = Rope::from(text);
        StyledFile::from(rope)
    }
}

impl SchemeCompatible for StyledFile {
    fn rtd() -> Arc<RecordTypeDescriptor>
    where
        Self: Sized
    {
        rtd!(name: "&StyledFile", sealed: true)
    }
}

#[bridge(name = "styled-file-prepend", lib = "(styled-text)")]
pub fn styled_file_prepend_segment(args: &[Value]) -> Result<Vec<Value>, Exception> {
    let Some((file, rest)) = args.split_first() else {
        return Err(Exception::wrong_num_of_args(3, args.len()));
    };
    let Some((line_no, rest)) = rest.split_first() else {
        return Err(Exception::wrong_num_of_args(3, args.len()));
    };
    let Some((text, _)) = rest.split_first() else {
        return Err(Exception::wrong_num_of_args(3, args.len()));
    };
    let file: Gc<StyledFile> = file.clone().try_to_rust_type()?;
    let mut file = (*file).clone();
    let line_no: SimpleNumber = line_no.clone().try_into()?;
    let line_no: i64 = match line_no {
        SimpleNumber::FixedInteger(line_no) => line_no,
        _ => return Err(Exception::error(String::from("Wrong kind of number for styled-file-prepend")))
    };
    let line_no = u64::from_ne_bytes(line_no.to_ne_bytes());
    let text: Gc<StyledText> = text.clone().try_to_rust_type()?;
    let text = (*text).clone();

    file.prepend_segment(line_no as usize, text);
    Ok(vec![Value::from(Record::from_rust_type(file))])
}

#[bridge(name = "styled-file-append", lib = "(styled-text)")]
pub fn styled_file_append_segment(args: &[Value]) -> Result<Vec<Value>, Exception> {
    let Some((file, rest)) = args.split_first() else {
        return Err(Exception::wrong_num_of_args(3, args.len()));
    };
    let Some((line_no, rest)) = rest.split_first() else {
        return Err(Exception::wrong_num_of_args(3, args.len()));
    };
    let Some((text, _)) = rest.split_first() else {
        return Err(Exception::wrong_num_of_args(3, args.len()));
    };
    let file: Gc<StyledFile> = file.clone().try_to_rust_type()?;
    let mut file = (*file).clone();
    let line_no: SimpleNumber = line_no.clone().try_into()?;
    let line_no: i64 = match line_no {
        SimpleNumber::FixedInteger(line_no) => line_no,
        _ => return Err(Exception::error(String::from("Wrong kind of number for styled-file-append")))
    };
    let line_no = u64::from_ne_bytes(line_no.to_ne_bytes());
    let text: Gc<StyledText> = text.clone().try_to_rust_type()?;
    let text = (*text).clone();

    file.append_segment(line_no as usize, text);
    Ok(vec![Value::from(Record::from_rust_type(file))])
}


#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash)]
pub enum ColorValue {
    Rgb {
        r: u8,
        g: u8,
        b: u8,
    },
    Ansi(u8),
}

impl ColorValue {
    pub fn from_hex(digit: u32) -> Self {
        ColorValue::Rgb {
            r: ((digit >> 16) & 0xFF) as u8,
            g: ((digit >> 8) & 0xFF) as u8,
            b: (digit & 0xFF) as u8,
        }
    }
}


/*
impl SmobData for ColorValue {
    fn print(&self) -> String {
        match self {
            ColorValue::Rgb { r, g, b } => {
                format!("#<rgb({},{},{})>", r, g, b)
            }
            ColorValue::Ansi(value) => {
                format!("#<Ansi({})>", value)
            }
        }
    }

    fn heap_size(&self) -> usize {
        0
    }

    fn eq(&self, other: SchemeObject) -> bool {
        let Some(other) = other.cast_smob(COLOR_VALUE_SMOB_TAG.clone()) else {
            return false;
        };
        *self == *other.borrow()
    }
}
*/

#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub struct ColorDefinition {
    color: ColorType,
    value: ColorValue,
}

impl ColorDefinition {
    pub fn new(color: ColorType, value: ColorValue) -> Self {
        Self { color, value }
    }

    pub fn to_tuple(self) -> (ColorType, ColorValue) {
        (self.color, self.value)
    }
}


