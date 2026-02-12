use std::borrow::Cow;
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

    fn process_segment(
        cursor_index: &mut usize,
        cursors: &[Cursor],
        line_index: usize,
        lines: &mut Vec<Vec<StyledText>>,
        current_line: &mut Vec<StyledText>,
        column_index: &mut usize,
        found_mark: &mut bool,
        found_cursor: &mut bool,
        text: TextChunk,
        fg_color: ColorType,
        bg_color: ColorType,
        attribute: TextAttribute,
    ) {
        let mut start = text.start();
        let mut current_pos = start;
        let mut prev_pos = start;

        for slice in text.graphemes() {
            let grapheme_len = slice.len();
            let next_pos = current_pos + grapheme_len;

            if *cursor_index < cursors.len() {
                let cursor = &cursors[*cursor_index];
                let at_cursor = *column_index == cursor.column() && line_index == cursor.line();
                let at_mark = cursor.is_mark_set()
                    && *column_index == cursor.mark_column().unwrap()
                    && line_index == cursor.mark_line().unwrap();

                // Handle finding the cursor position
                if at_cursor {
                    *found_cursor = true;
                    if !*found_mark {
                        // Push text before cursor (if any)
                        if start < current_pos {
                            current_line.push(StyledText::Style {
                                fg_color,
                                bg_color,
                                attribute,
                                text: TextChunk::new(text.rope.clone(), start, current_pos),
                            });
                        }
                        start = current_pos;
                    }
                }
                // Handle finding the mark position
                else if at_mark {
                    *found_mark = true;
                    if !*found_cursor {
                        // Push text before mark (if any)
                        if start < current_pos {
                            current_line.push(StyledText::Style {
                                fg_color,
                                bg_color,
                                attribute,
                                text: TextChunk::new(text.rope.clone(), start, current_pos),
                            });
                        }
                        start = current_pos;
                    } else if *column_index != cursor.column() + 1 {
                        // Push selected text
                        if start < current_pos {
                            current_line.push(StyledText::Style {
                                bg_color: ColorType::Selection,
                                fg_color,
                                attribute,
                                text: TextChunk::new(text.rope.clone(), start, current_pos),
                            });
                        }

                        // Push current grapheme with selection
                        current_line.push(StyledText::Style {
                            bg_color: ColorType::Selection,
                            fg_color,
                            attribute,
                            text: TextChunk::new(text.rope.clone(), current_pos, next_pos),
                        });

                        start = next_pos;
                        *found_mark = false;
                        *found_cursor = false;
                        *cursor_index += 1;
                        current_pos = next_pos;
                        *column_index += 1;
                        continue;
                    }
                }

                // Handle position after cursor (cursor highlight)
                if *found_cursor
                    && *column_index == cursor.column() + 1
                    && line_index == cursor.line()
                {
                    let cursor_color = if cursor.is_main() {
                        ColorType::Cursor
                    } else {
                        ColorType::SecondaryCursor
                    };

                    if cursor.is_mark_and_cursor_same() {
                        *found_mark = false;
                    }

                    if *found_mark {
                        // Push selected text before cursor grapheme
                        if start < prev_pos {
                            current_line.push(StyledText::Style {
                                bg_color: ColorType::Selection,
                                fg_color,
                                attribute: TextAttribute::empty(),
                                text: TextChunk::new(text.rope.clone(), start, prev_pos),
                            });
                        }

                        // Push cursor grapheme (from prev_pos to current_pos)
                        current_line.push(StyledText::Style {
                            bg_color: cursor_color,
                            fg_color,
                            attribute: TextAttribute::empty(),
                            text: TextChunk::new(text.rope.clone(), prev_pos, current_pos),
                        });

                        start = current_pos;
                    } else {
                        // Push cursor grapheme (from prev_pos to current_pos)
                        current_line.push(StyledText::Style {
                            bg_color: cursor_color,
                            fg_color,
                            attribute: TextAttribute::empty(),
                            text: TextChunk::new(text.rope.clone(), prev_pos, current_pos),
                        });

                        start = current_pos;
                    }

                    if *found_cursor && !cursor.is_mark_set() {
                        *cursor_index += 1;
                    } else if *found_cursor && cursor.is_mark_set() && *found_mark {
                        *cursor_index += 1;
                        *found_cursor = false;
                        *found_mark = false;
                    }
                }
                // Handle cursor at newline
                else if *found_cursor
                    && slice.contains('\n')
                    && *column_index == cursor.column()
                    && line_index == cursor.line()
                {
                    let cursor_color = if cursor.is_main() {
                        ColorType::Cursor
                    } else {
                        ColorType::SecondaryCursor
                    };

                    if cursor.is_mark_and_cursor_same() {
                        *found_mark = false;
                    }

                    if *found_mark {
                        // Push selected text
                        if start < current_pos {
                            current_line.push(StyledText::Style {
                                bg_color: ColorType::Selection,
                                fg_color,
                                attribute: TextAttribute::empty(),
                                text: TextChunk::new(text.rope.clone(), start, current_pos),
                            });
                        }

                        // Push cursor (space at newline)
                        current_line.push(StyledText::Style {
                            bg_color: cursor_color,
                            fg_color,
                            attribute: TextAttribute::empty(),
                            text: TextChunk::from(String::from(' ')),
                        });
                    } else {
                        // Push text before cursor
                        if start < current_pos {
                            current_line.push(StyledText::Style {
                                bg_color: ColorType::Selection,
                                fg_color,
                                attribute: TextAttribute::empty(),
                                text: TextChunk::new(text.rope.clone(), start, current_pos),
                            });
                        }

                        // Push cursor (space at newline)
                        current_line.push(StyledText::Style {
                            bg_color: cursor_color,
                            fg_color,
                            attribute: TextAttribute::empty(),
                            text: TextChunk::from(String::from(' ')),
                        });
                    }

                    start = current_pos;

                    if *found_cursor && !cursor.is_mark_set() {
                        *cursor_index += 1;
                    } else if *found_cursor && cursor.is_mark_set() && *found_mark {
                        *cursor_index += 1;
                        *found_cursor = false;
                        *found_mark = false;
                    }
                }
                // Handle mark position after cursor
                else if *found_mark
                    && *column_index == cursor.mark_column().unwrap()
                    && line_index == cursor.mark_line().unwrap()
                {
                    // Push selected text
                    if start < current_pos {
                        current_line.push(StyledText::Style {
                            bg_color: ColorType::Selection,
                            fg_color,
                            attribute,
                            text: TextChunk::new(text.rope.clone(), start, current_pos),
                        });
                    }
                    start = current_pos;
                }
            }

            prev_pos = current_pos;
            current_pos = next_pos;
            *column_index += 1;
        }

        // Push any remaining text at the end
        if start < current_pos {
            if *found_mark && !*found_cursor {
                current_line.push(StyledText::Style {
                    bg_color: ColorType::Selection,
                    fg_color,
                    attribute,
                    text: TextChunk::new(text.rope.clone(), start, current_pos),
                });
            } else if *found_cursor
                && *cursor_index < cursors.len()
                && cursors[*cursor_index].is_mark_set()
                && !cursors[*cursor_index].is_mark_and_cursor_same()
                && !*found_mark
            {
                current_line.push(StyledText::Style {
                    bg_color: ColorType::Selection,
                    fg_color,
                    attribute,
                    text: TextChunk::new(text.rope.clone(), start, current_pos),
                });
            } else {
                current_line.push(StyledText::Style {
                    fg_color,
                    bg_color,
                    attribute,
                    text: TextChunk::new(text.rope.clone(), start, current_pos),
                });
            }
        }
    }


    /// Cursors must be in order they are logically in the file
    pub fn place_cursors(self, cursors: &[Cursor]) -> Self {
        let mut cursor_index = 0;
        let mut lines = Vec::new();
        let mut found_mark = false;
        let mut found_cursor = false;
        //let prepend_line = major_mode.read().prepend_line();
        //let append_line = major_mode.read().append_line();
        for (line_index, line) in self.lines.into_iter().enumerate() {
            let mut current_line = Vec::new();
            let mut column_index = 0;
            /*if let Some(ref proc) = prepend_line {
                let args = &[
                    Value::from(Number::from(line_index)),
                    Value::from(Number::from(total_lines))
                ];
                let value = proc.call(args).await.unwrap();
                if value.len() != 0 {
                    let text: Gc<StyledText> = value[0].clone().try_to_rust_type().unwrap();
                    current_line.push(text.read().clone());
                }
            }*/
            for segment in line {
                match segment {
                    StyledText::None { text } => {
                        //println!("cursor index: {}, cursor-count: {}", cursor_index, cursors.len());
                        Self::process_segment(
                            &mut cursor_index,
                            cursors,
                            line_index,
                            &mut lines,
                            &mut current_line,
                            &mut column_index,
                            &mut found_mark,
                            &mut found_cursor,
                            text,
                            ColorType::Text,
                            ColorType::Base,
                            TextAttribute::empty(),
                        );
                    }
                    StyledText::Style {
                        fg_color,
                        bg_color,
                        attribute,
                        text,
                    } => {
                        Self::process_segment(
                            &mut cursor_index,
                            cursors,
                            line_index,
                            &mut lines,
                            &mut current_line,
                            &mut column_index,
                            &mut found_mark,
                            &mut found_cursor,
                            text,
                            fg_color,
                            bg_color,
                            attribute,
                        );
                    }
                }
            }
            /*if let Some(ref proc) = append_line {
                let args = &[
                    Value::from(Number::from(line_index)),
                    Value::from(Number::from(total_lines))
                ];
                let value = proc.call(args).await.unwrap();
                if value.len() != 0 {
                    let text: Gc<StyledText> = value[0].clone().try_to_rust_type().unwrap();
                    current_line.push(text.read().clone());
                }
            }*/
            lines.push(current_line);
        }
        Self {
            lines,
        }
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


