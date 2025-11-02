use std::sync::{Arc};
use bitflags::bitflags;
use scheme_rs::exceptions::Condition;
use scheme_rs::gc::{Gc, Trace};
use scheme_rs::num::Number;
use scheme_rs::records::{rtd, Record, RecordTypeDescriptor, SchemeCompatible};
use scheme_rs::registry::bridge;
use scheme_rs::value::Value;
use crate::kernel::buffer::Cursor;

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


#[derive(Debug, Clone, Eq, PartialEq, Hash, Trace)]
pub enum StyledText {
    None(String),
    Style {
        fg_color: ColorType,
        bg_color: ColorType,
        attribute: Arc<TextAttribute>,
        text: String,
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

        let text = StyledText::Style {
            fg_color,
            bg_color,
            attribute: Arc::new(attributes),
            text,
        };
        Ok(vec![Value::from(Record::from_rust_type(text))])

    } else {
        let text = StyledText::None(text);
        Ok(vec![Value::from(Record::from_rust_type(text))])
    }
}

#[derive(Debug, Clone, Eq, PartialEq, Hash, Trace)]
pub struct StyledFile {
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
    
    /// Cursors must be in order they are logically in the file
    pub fn place_cursors(self, cursors: &[Cursor]) -> Self {
        let mut cursor_index = 0;
        let mut lines = Vec::new();
        let mut mark_active = false;
        for (line_index, line) in self.lines.into_iter().enumerate() {
            let mut current_line = Vec::new();
            let mut column_index = 0;
            for segment in line {
                match segment {
                    StyledText::None(text) => {
                        let mut buffer = String::new();
                        for ch in text.chars() {
                            if cursor_index < cursors.len() {
                                if column_index == cursors[cursor_index].column()
                                    && line_index == cursors[cursor_index].line() {
                                    current_line.push(StyledText::None(buffer));
                                    buffer = String::new();
                                } else if cursors[cursor_index].is_mark_set() 
                                    && (column_index == cursors[cursor_index].mark_column().unwrap()
                                    && line_index == cursors[cursor_index].mark_line().unwrap()) {
                                    mark_active = true;
                                    current_line.push(StyledText::None(buffer));
                                    buffer = String::new();
                                }
                                if column_index == cursors[cursor_index].column() + 1
                                    && line_index == cursors[cursor_index].line() {
                                    if mark_active {
                                        if cursors[cursor_index].is_mark_and_cursor_same() {
                                            current_line.push(StyledText::Style {
                                                bg_color: ColorType::Cursor,
                                                fg_color: ColorType::Text,
                                                attribute: Arc::new(TextAttribute::empty()),
                                                text: buffer,
                                            });
                                        } else {
                                            let cursor_char = buffer.pop().unwrap();
                                            current_line.push(StyledText::Style {
                                                bg_color: ColorType::Selection,
                                                fg_color: ColorType::Text,
                                                attribute: Arc::new(TextAttribute::empty()),
                                                text: buffer,
                                            });
                                            current_line.push(StyledText::Style {
                                                bg_color: ColorType::Cursor,
                                                fg_color: ColorType::Text,
                                                attribute: Arc::new(TextAttribute::empty()),
                                                text: String::from(cursor_char),
                                            })
                                        }
                                        cursor_index += 1;
                                    } else {
                                        current_line.push(StyledText::Style {
                                            bg_color: ColorType::Cursor,
                                            fg_color: ColorType::Text,
                                            attribute: Arc::new(TextAttribute::empty()),
                                            text: buffer,
                                        });
                                        cursor_index += 1;
                                    }
                                    buffer = String::new();
                                }
                            }
                            if ch == '\n' && cursor_index < cursors.len() {
                                if column_index == cursors[cursor_index].column()
                                    && line_index == cursors[cursor_index].line() {
                                    if mark_active {
                                        if cursors[cursor_index].is_mark_and_cursor_same() {
                                            current_line.push(StyledText::None(buffer));
                                            buffer = String::new();
                                            
                                            current_line.push(StyledText::Style {
                                                bg_color: ColorType::Cursor,
                                                fg_color: ColorType::Text,
                                                attribute: Arc::new(TextAttribute::empty()),
                                                text: String::from(' '),
                                            });
                                        } else {
                                            current_line.push(StyledText::Style {
                                                bg_color: ColorType::Selection,
                                                fg_color: ColorType::Text,
                                                attribute: Arc::new(TextAttribute::empty()),
                                                text: buffer,
                                            });
                                            current_line.push(StyledText::Style {
                                                bg_color: ColorType::Cursor,
                                                fg_color: ColorType::Text,
                                                attribute: Arc::new(TextAttribute::empty()),
                                                text: String::from(' '),
                                            });

                                            buffer = String::new();
                                        }
                                        cursor_index += 1;
                                    } else {
                                        current_line.push(StyledText::Style {
                                            bg_color: ColorType::Cursor,
                                            fg_color: ColorType::Text,
                                            attribute: Arc::new(TextAttribute::empty()),
                                            text: String::from(' '),
                                        });
                                        cursor_index += 1;
                                    }
                                }
                            }
                            buffer.push(ch);
                            column_index += 1;
                        }
                        current_line.push(StyledText::None(buffer));
                    }
                    StyledText::Style {
                        fg_color,
                        bg_color,
                        attribute,
                        text,
                    } => {
                        let mut buffer = String::new();
                        for ch in text.chars() {
                            if cursor_index < cursors.len() {
                                if column_index == cursors[cursor_index].column()
                                    && line_index == cursors[cursor_index].line() {
                                    current_line.push(StyledText::Style {
                                        fg_color,
                                        bg_color,
                                        attribute: attribute.clone(),
                                        text: buffer,
                                    });
                                    buffer = String::new();
                                }
                                if column_index == cursors[cursor_index].column() + 1
                                    && line_index == cursors[cursor_index].line() {
                                    cursor_index += 1;
                                    current_line.push(StyledText::Style {
                                        bg_color: ColorType::Cursor,
                                        fg_color,
                                        attribute: attribute.clone(),
                                        text: buffer,
                                    });
                                    buffer = String::new();
                                }
                            }
                            if ch == '\n' {
                                current_line.push(StyledText::Style {
                                    fg_color,
                                    bg_color,
                                    attribute: attribute.clone(),
                                    text: buffer,
                                });
                                buffer = String::new();
                                if cursor_index < cursors.len() {
                                    if column_index == cursors[cursor_index].column()
                                        && line_index == cursors[cursor_index].line() {
                                        cursor_index += 1;
                                        current_line.push(StyledText::Style {
                                            bg_color: ColorType::Cursor,
                                            fg_color: ColorType::Text,
                                            attribute: Arc::new(TextAttribute::empty()),
                                            text: String::from(' '),
                                        });
                                    }
                                }
                            }
                            buffer.push(ch);
                            column_index += 1;
                            //index += ch.len_utf8();
                        }
                        current_line.push(StyledText::Style {
                            fg_color,
                            bg_color,
                            attribute: attribute.clone(),
                            text: buffer,
                        });
                    }
                }
            }
            lines.push(current_line);
        }
        Self {
            lines,
        }
    }
    
}

impl From<String> for StyledFile {
    fn from(text: String) -> Self {
        Self {
            lines: text.lines().map(ToString::to_string).map(|mut x| {x.push('\n'); x}).map(StyledText::None).map(|x| vec![x]).collect(),
        }
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



