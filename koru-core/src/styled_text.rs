use std::fmt::Display;
use std::sync::{Arc};
use bitflags::bitflags;
use crop::Rope;
use scheme_rs::exceptions::Condition;
use scheme_rs::gc::{Gc, Trace};
use scheme_rs::num::Number;
use scheme_rs::records::{rtd, Record, RecordTypeDescriptor, SchemeCompatible};
use scheme_rs::registry::bridge;
use scheme_rs::value::Value;
use crate::kernel::buffer::Cursor;
use crate::kernel::scheme_api::major_mode::MajorMode;

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
        rtd!(name: "&TextChunk")
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
        rtd!(name: "&ColorType")
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
        rtd!(name: "&StyledText")
    }
}

#[bridge(name = "styled-text-create", lib = "(styled-text)")]
pub fn styled_text_create(args: &[Value]) -> Result<Vec<Value>, Condition> {
    let Some((text, rest)) = args.split_first() else {
        return Err(Condition::wrong_num_of_args(1, args.len()));
    };
    let text: String = text.clone().try_into()?;
    if let Some((fg_color, rest)) = rest.split_first() {
        let fg_color: String = fg_color.clone().try_into()?;
        let Some((bg_color, rest)) = rest.split_first() else {
            return Err(Condition::wrong_num_of_args(3, args.len()));
        };
        let bg_color: String = bg_color.clone().try_into()?;
        let fg_color = match fg_color.as_str().try_into()  {
            Ok(color) => color,
            Err(msg) => {
                return Err(Condition::error(msg));
            }
        };
        let bg_color = match bg_color.as_str().try_into()  {
            Ok(color) => color,
            Err(msg) => {
                return Err(Condition::error(msg));
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
                    return Err(Condition::error(String::from("attribute is not one of 'italic, 'bold, 'strikethrough, or 'underline")));
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
    rope: Rope,
    lines: Vec<Vec<StyledText>>,
}

impl StyledFile {
    pub fn new(rope: Rope) -> Self {
        Self {
            rope,
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
    
    /// Cursors must be in order they are logically in the file
    pub async fn place_cursors(self, cursors: &[Cursor], major_mode: Gc<MajorMode>) -> Self {
        let total_lines = self.lines.len();
        let mut cursor_index = 0;
        let mut lines = Vec::new();
        let mut found_mark = false;
        let mut found_cursor = false;
        let prepend_line = major_mode.read().prepend_line();
        let append_line = major_mode.read().append_line();
        for (line_index, line) in self.lines.into_iter().enumerate() {
            let mut current_line = Vec::new();
            let mut column_index = 0;
            if let Some(ref proc) = prepend_line {
                let args = &[
                    Value::from(Number::from(line_index)), 
                    Value::from(Number::from(total_lines))
                ];
                let value = proc.call(args).await.unwrap();
                if value.len() != 0 {
                    let text: Gc<StyledText> = value[0].clone().try_into_rust_type().unwrap();
                    current_line.push(text.read().clone());
                }
            }
            for segment in line {
                match segment {
                    StyledText::None { text } => {
                        let mut start = text.start();
                        let mut end = start;
                        for ch in text.chars() {
                            if cursor_index < cursors.len() {
                                if column_index == cursors[cursor_index].column()
                                    && line_index == cursors[cursor_index].line() {
                                    found_cursor = true;
                                    if !found_mark {
                                        current_line.push(StyledText::None { text: TextChunk::new(self.rope.clone(), start, end) });
                                        start = end;
                                    }

                                } else if cursors[cursor_index].is_mark_set()
                                    && column_index == cursors[cursor_index].mark_column().unwrap()
                                    && line_index == cursors[cursor_index].mark_line().unwrap() {
                                    found_mark = true;
                                    if !found_cursor {
                                        current_line.push(StyledText::None { text: TextChunk::new(self.rope.clone(), start, end) });
                                        start = end;
                                    } else {
                                        current_line.push(StyledText::Style {
                                            bg_color: ColorType::Selection,
                                            fg_color: ColorType::Text,
                                            attribute: TextAttribute::empty(),
                                            text: TextChunk::new(self.rope.clone(), start, end),
                                        });
                                        start = end;
                                        found_mark = false;
                                        found_cursor = false;
                                        cursor_index += 1;
                                        column_index += 1;
                                        continue;
                                    }
                                }
                                if (found_cursor
                                    && column_index == cursors[cursor_index].column() + 1
                                    && line_index == cursors[cursor_index].line())
                                    || (found_cursor
                                        && ch == '\n'
                                        && column_index == cursors[cursor_index].column()
                                        && line_index == cursors[cursor_index].line()) {
                                    

                                    if cursors[cursor_index].is_mark_and_cursor_same() {
                                        found_mark = false;
                                    }

                                    if found_mark {
                                        if ch == '\n'
                                            && column_index == cursors[cursor_index].column()
                                            && line_index == cursors[cursor_index].line() {
                                            current_line.push(StyledText::Style {
                                                bg_color: ColorType::Selection,
                                                fg_color: ColorType::Text,
                                                attribute: TextAttribute::empty(),
                                                text: TextChunk::new(self.rope.clone(), start, end),
                                            });
                                            start = end;
                                            
                                            current_line.push(StyledText::Style {
                                                bg_color: ColorType::Cursor,
                                                fg_color: ColorType::Text,
                                                attribute: TextAttribute::empty(),
                                                text: TextChunk::from(String::from(' ')),
                                            });
                                        } else {
                                            current_line.push(StyledText::Style {
                                                bg_color: ColorType::Selection,
                                                fg_color: ColorType::Text,
                                                attribute: TextAttribute::empty(),
                                                text: TextChunk::new(self.rope.clone(), start, end - ch.len_utf8()),
                                            });
                                            start = end - ch.len_utf8();
                                            current_line.push(StyledText::Style {
                                                bg_color: ColorType::Cursor,
                                                fg_color: ColorType::Text,
                                                attribute: TextAttribute::empty(),
                                                text: TextChunk::new(self.rope.clone(), start, end),
                                            });
                                            start = end;
                                        }                                        
                                    } else {
                                        if ch == '\n'
                                            && column_index == cursors[cursor_index].column()
                                            && line_index == cursors[cursor_index].line() {
                                            current_line.push(StyledText::Style {
                                                bg_color: ColorType::Selection,
                                                fg_color: ColorType::Text,
                                                attribute: TextAttribute::empty(),
                                                text: TextChunk::new(self.rope.clone(), start, end),
                                            });
                                            start = end;

                                            current_line.push(StyledText::Style {
                                                bg_color: ColorType::Cursor,
                                                fg_color: ColorType::Text,
                                                attribute: TextAttribute::empty(),
                                                text: TextChunk::from(String::from(' ')),
                                            });
                                        } else {
                                            current_line.push(StyledText::Style {
                                                bg_color: ColorType::Cursor,
                                                fg_color: ColorType::Text,
                                                attribute: TextAttribute::empty(),
                                                text: TextChunk::new(self.rope.clone(), start, end),
                                            });
                                            start = end;
                                        }
                                    }
                                    if !cursors[cursor_index].is_mark_set() {
                                        cursor_index += 1;
                                    }
                                } else if found_mark
                                    && column_index == cursors[cursor_index].mark_column().unwrap()
                                    && line_index == cursors[cursor_index].mark_line().unwrap() {
                                    current_line.push(StyledText::Style {
                                        bg_color: ColorType::Selection,
                                        fg_color: ColorType::Text,
                                        attribute: TextAttribute::empty(),
                                        text: TextChunk::new(self.rope.clone(), start, end),
                                    });
                                    start = end;
                                }
                            }
                            end += ch.len_utf8();
                            column_index += 1;
                        }
                        if found_mark && !found_cursor {
                            current_line.push(StyledText::Style {
                                bg_color: ColorType::Selection,
                                fg_color: ColorType::Text,
                                attribute: TextAttribute::empty(),
                                text: TextChunk::new(self.rope.clone(), start, end),
                            });
                            start = end;
                        } else if found_cursor && (!found_mark && cursor_index < cursors.len() && cursors[cursor_index].is_mark_set() && !cursors[cursor_index].is_mark_and_cursor_same()) {
                            current_line.push(StyledText::Style {
                                bg_color: ColorType::Selection,
                                fg_color: ColorType::Text,
                                attribute: TextAttribute::empty(),
                                text: TextChunk::new(self.rope.clone(), start, end),
                            });
                            start = end;
                        } else {
                            current_line.push(StyledText::None {
                                text: TextChunk::new(self.rope.clone(), start, end),
                            });
                            start = end;
                        }
                    }
                    StyledText::Style {
                        fg_color,
                        bg_color,
                        attribute,
                        text,
                    } => {
                        let mut start = text.start();
                        let mut end = start;
                        for ch in text.chars() {
                            if cursor_index < cursors.len() {
                                if column_index == cursors[cursor_index].column()
                                    && line_index == cursors[cursor_index].line() {
                                    found_cursor = true;
                                    if !found_mark {
                                        current_line.push(StyledText::Style {
                                            fg_color,
                                            bg_color,
                                            attribute,
                                            text: TextChunk::new(self.rope.clone(), start, end) 
                                        });
                                        start = end;
                                    }

                                } else if cursors[cursor_index].is_mark_set()
                                    && column_index == cursors[cursor_index].mark_column().unwrap()
                                    && line_index == cursors[cursor_index].mark_line().unwrap() {
                                    found_mark = true;
                                    if !found_cursor {
                                        current_line.push(StyledText::Style {
                                            fg_color,
                                            bg_color,
                                            attribute,
                                            text: TextChunk::new(self.rope.clone(), start, end)
                                        });
                                        start = end;
                                    } else {
                                        current_line.push(StyledText::Style {
                                            bg_color: ColorType::Selection,
                                            fg_color,
                                            attribute,
                                            text: TextChunk::new(self.rope.clone(), start, end),
                                        });
                                        start = end;
                                        found_mark = false;
                                        found_cursor = false;
                                        cursor_index += 1;
                                        column_index += 1;
                                        continue;
                                    }
                                }
                                if (found_cursor
                                    && column_index == cursors[cursor_index].column() + 1
                                    && line_index == cursors[cursor_index].line())
                                    || (found_cursor
                                    && ch == '\n'
                                    && column_index == cursors[cursor_index].column()
                                    && line_index == cursors[cursor_index].line()) {


                                    if cursors[cursor_index].is_mark_and_cursor_same() {
                                        found_mark = false;
                                    }

                                    if found_mark {
                                        if ch == '\n'
                                            && column_index == cursors[cursor_index].column()
                                            && line_index == cursors[cursor_index].line() {
                                            current_line.push(StyledText::Style {
                                                bg_color: ColorType::Selection,
                                                fg_color,
                                                attribute,
                                                text: TextChunk::new(self.rope.clone(), start, end),
                                            });
                                            start = end;

                                            current_line.push(StyledText::Style {
                                                bg_color: ColorType::Cursor,
                                                fg_color,
                                                attribute,
                                                text: TextChunk::from(String::from(' ')),
                                            });
                                        } else {
                                            current_line.push(StyledText::Style {
                                                bg_color: ColorType::Selection,
                                                fg_color,
                                                attribute,
                                                text: TextChunk::new(self.rope.clone(), start, end - ch.len_utf8()),
                                            });
                                            start = end - ch.len_utf8();
                                            current_line.push(StyledText::Style {
                                                bg_color: ColorType::Cursor,
                                                fg_color,
                                                attribute,
                                                text: TextChunk::new(self.rope.clone(), start, end),
                                            });
                                            start = end;
                                        }
                                    } else {
                                        if ch == '\n'
                                            && column_index == cursors[cursor_index].column()
                                            && line_index == cursors[cursor_index].line() {
                                            current_line.push(StyledText::Style {
                                                bg_color: ColorType::Selection,
                                                fg_color,
                                                attribute,
                                                text: TextChunk::new(self.rope.clone(), start, end),
                                            });
                                            start = end;

                                            current_line.push(StyledText::Style {
                                                bg_color: ColorType::Cursor,
                                                fg_color: ColorType::Text,
                                                attribute,
                                                text: TextChunk::from(String::from(' ')),
                                            });
                                        } else {
                                            current_line.push(StyledText::Style {
                                                bg_color: ColorType::Cursor,
                                                fg_color,
                                                attribute,
                                                text: TextChunk::new(self.rope.clone(), start, end),
                                            });
                                            start = end;
                                        }
                                    }
                                    if !cursors[cursor_index].is_mark_set() {
                                        cursor_index += 1;
                                    }
                                } else if found_mark
                                    && column_index == cursors[cursor_index].mark_column().unwrap()
                                    && line_index == cursors[cursor_index].mark_line().unwrap() {
                                    current_line.push(StyledText::Style {
                                        bg_color: ColorType::Selection,
                                        fg_color,
                                        attribute,
                                        text: TextChunk::new(self.rope.clone(), start, end),
                                    });
                                    start = end;
                                }
                            }
                            end += ch.len_utf8();
                            column_index += 1;
                        }
                        if found_mark && !found_cursor {
                            current_line.push(StyledText::Style {
                                bg_color: ColorType::Selection,
                                fg_color,
                                attribute,
                                text: TextChunk::new(self.rope.clone(), start, end),
                            });
                            start = end;
                        } else if found_cursor && (!found_mark && cursor_index < cursors.len() && cursors[cursor_index].is_mark_set() && !cursors[cursor_index].is_mark_and_cursor_same()) {
                            current_line.push(StyledText::Style {
                                bg_color: ColorType::Selection,
                                fg_color,
                                attribute,
                                text: TextChunk::new(self.rope.clone(), start, end),
                            });
                            start = end;
                        } else {
                            current_line.push(StyledText::Style {
                                fg_color,
                                bg_color,
                                attribute,
                                text: TextChunk::new(self.rope.clone(), start, end)
                            });
                            start = end;
                        }
                    }
                }
            }
            if let Some(ref proc) = append_line {
                let args = &[
                    Value::from(Number::from(line_index)),
                    Value::from(Number::from(total_lines))
                ];
                let value = proc.call(args).await.unwrap();
                if value.len() != 0 {
                    let text: Gc<StyledText> = value[0].clone().try_into_rust_type().unwrap();
                    current_line.push(text.read().clone());
                }
            }
            lines.push(current_line);
        }
        Self {
            rope: self.rope,
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

        Self {
            rope: text,
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
        rtd!(name: "&StyledFile")
    }
}

#[bridge(name = "styled-file-prepend", lib = "(styled-text)")]
pub fn styled_file_prepend_segment(args: &[Value]) -> Result<Vec<Value>, Condition> {
    let Some((file, rest)) = args.split_first() else {
        return Err(Condition::wrong_num_of_args(3, args.len()));
    };
    let Some((line_no, rest)) = rest.split_first() else {
        return Err(Condition::wrong_num_of_args(3, args.len()));
    };
    let Some((text, _)) = rest.split_first() else {
        return Err(Condition::wrong_num_of_args(3, args.len()));
    };
    let file: Gc<StyledFile> = file.clone().try_into_rust_type()?;
    let line_no: Arc<Number> = line_no.clone().try_into()?;
    let line_no: i64 = match line_no.as_ref() {
        Number::FixedInteger(line_no) => *line_no,
        _ => return Err(Condition::error(String::from("Wrong kind of number for styled-text-prepend-segment")))
    };
    let line_no = u64::from_ne_bytes(line_no.to_ne_bytes());
    let text: Gc<StyledText> = text.clone().try_into_rust_type()?;
    let text = text.read().clone();
    
    file.write().prepend_segment(line_no as usize, text);
    Ok(vec![])
}

#[bridge(name = "styled-file-append", lib = "(styled-text)")]
pub fn styled_file_append_segment(args: &[Value]) -> Result<Vec<Value>, Condition> {
    let Some((file, rest)) = args.split_first() else {
        return Err(Condition::wrong_num_of_args(3, args.len()));
    };
    let Some((line_no, rest)) = rest.split_first() else {
        return Err(Condition::wrong_num_of_args(3, args.len()));
    };
    let Some((text, _)) = rest.split_first() else {
        return Err(Condition::wrong_num_of_args(3, args.len()));
    };
    let file: Gc<StyledFile> = file.clone().try_into_rust_type()?;
    let line_no: Arc<Number> = line_no.clone().try_into()?;
    let line_no: i64 = match line_no.as_ref() {
        Number::FixedInteger(line_no) => *line_no,
        _ => return Err(Condition::error(String::from("Wrong kind of number for styled-text-prepend-segment")))
    };
    let line_no = u64::from_ne_bytes(line_no.to_ne_bytes());
    let text: Gc<StyledText> = text.clone().try_into_rust_type()?;
    let text = text.read().clone();

    file.write().append_segment(line_no as usize, text);
    Ok(vec![])
}


#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub enum ColorValue {
    Rgb {
        r: u8,
        g: u8,
        b: u8,
    },
    Ansi(u8),
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



