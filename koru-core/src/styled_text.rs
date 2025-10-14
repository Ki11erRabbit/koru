use bitflags::bitflags;
use mlua::{AnyUserData, UserData, UserDataMethods};
use crate::kernel::cursor::Cursor;

bitflags! {
    #[derive(Debug, Copy, Clone, Eq, PartialEq, Hash)]
    pub struct TextAttribute: u8 {
        const Italic = 0b0000_0001;
        const Bold = 0b0000_0010;
        const Strikethrough = 0b0000_0100;
        const Underline = 0b0000_1000;
    }
}

#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash)]
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


#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub enum StyledText {
    None(String),
    Style {
        fg_color: ColorType,
        bg_color: ColorType,
        attribute: TextAttribute,
        text: String,
    }
}

impl UserData for StyledText {
    
}

#[derive(Debug, Clone, Eq, PartialEq, Hash)]
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
    
    pub fn prepend_segment(&mut self, line: usize, text: StyledText) {
        self.lines[line].insert(0, text);
    }
    
    pub fn append_segment(&mut self, line: usize, text: StyledText) {
        self.lines[line].push(text);
    }
    
    /// Cursors must be in order they are logically in the file
    pub fn place_cursors(self, cursors: &[Cursor]) -> Self {
        let mut index = 0;
        let mut cursor_index = 0;
        let mut lines = Vec::new();
        for line in self.lines {
            let mut current_line = Vec::new();
            for segment in line {
                match segment {
                    StyledText::None(text) => {
                        let mut buffer = String::new();
                        for ch in text.chars() {
                            if cursor_index < cursors.len() {
                                if index == cursors[cursor_index].start() {
                                    current_line.push(StyledText::None(buffer));
                                    buffer = String::new();
                                }
                                if index == cursors[cursor_index].stop() {
                                    cursor_index += 1;
                                    current_line.push(StyledText::Style {
                                        bg_color: ColorType::Cursor,
                                        fg_color: ColorType::Text,
                                        attribute: TextAttribute::empty(),
                                        text: buffer,
                                    });
                                    buffer = String::new();
                                }
                            }
                            buffer.push(ch);
                            index += 1;
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
                                if index == cursors[cursor_index].start() {
                                    current_line.push(StyledText::Style {
                                        fg_color,
                                        bg_color,
                                        attribute,
                                        text: buffer,
                                    });
                                    buffer = String::new();
                                }
                                if index == cursors[cursor_index].stop() {
                                    cursor_index += 1;
                                    current_line.push(StyledText::Style {
                                        bg_color: ColorType::Cursor,
                                        fg_color,
                                        attribute,
                                        text: buffer,
                                    });
                                    buffer = String::new();
                                }
                            }
                            
                            buffer.push(ch);
                            index += 1;
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

impl UserData for StyledFile {
    fn add_methods<M: UserDataMethods<Self>>(methods: &mut M) {
        methods.add_method_mut(
            "prepend_segment",
            |lua, this, (line, text): (mlua::Integer, AnyUserData)| {
                let text = text.take::<StyledText>()?;
                let line = line as usize;
                this.prepend_segment(line, text);
                Ok(())
            }
        )
    }
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

#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub struct ColorDefinition {
    color: ColorType,
    value: ColorValue,
}