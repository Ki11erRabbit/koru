use std::sync::{Arc};
use bitflags::bitflags;
use mlua::{AnyUserData, Lua, Table, UserData, UserDataMethods};
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

impl UserData for StyledText {
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
        for (line_index, line) in self.lines.into_iter().enumerate() {
            let mut current_line = Vec::new();
            let mut column_index = 0;
            for segment in line {
                match segment {
                    StyledText::None(text) => {
                        let mut buffer = String::new();
                        for ch in text.chars() {
                            if cursor_index < cursors.len() {
                                if column_index == cursors[cursor_index].column_start()
                                    && line_index == cursors[cursor_index].line_start() {
                                    current_line.push(StyledText::None(buffer));
                                    buffer = String::new();
                                }
                                if column_index == cursors[cursor_index].column_end()
                                    && line_index == cursors[cursor_index].line_end() {
                                    cursor_index += 1;
                                    current_line.push(StyledText::Style {
                                        bg_color: ColorType::Cursor,
                                        fg_color: ColorType::Text,
                                        attribute: Arc::new(TextAttribute::empty()),
                                        text: buffer,
                                    });
                                    buffer = String::new();
                                }
                            }
                            if ch == '\n' {
                                current_line.push(StyledText::None(buffer));
                                buffer = String::new();
                                if cursor_index < cursors.len() {
                                    if column_index == cursors[cursor_index].column_start()
                                        && line_index == cursors[cursor_index].line_end() {
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
                                if column_index == cursors[cursor_index].column_start()
                                    && line_index == cursors[cursor_index].line_start() {
                                    current_line.push(StyledText::Style {
                                        fg_color,
                                        bg_color,
                                        attribute: attribute.clone(),
                                        text: buffer,
                                    });
                                    buffer = String::new();
                                }
                                if column_index == cursors[cursor_index].column_end()
                                    && line_index == cursors[cursor_index].line_end() {
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
                                    if column_index == cursors[cursor_index].column_start()
                                        && line_index == cursors[cursor_index].line_end() {
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


impl UserData for StyledFile {
    fn add_methods<M: UserDataMethods<Self>>(methods: &mut M) {
        methods.add_method_mut(
            "prepend_segment",
            |_, this, (line, text): (mlua::Integer, AnyUserData)| {
                let text = text.take::<StyledText>()?;
                let line = line as usize;
                this.prepend_segment(line, text);
                Ok(())
            }
        );
        methods.add_method_mut(
            "append_segment",
            |_, this, (line, text): (mlua::Integer, AnyUserData)| {
                let line = line as usize;
                let text = text.take::<StyledText>()?;
                this.append_segment(line, text);
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

impl UserData for ColorValue {

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

impl UserData for ColorDefinition {

}


pub fn styled_text_module(lua: &Lua) -> mlua::Result<Table> {
    let exports = lua.create_table()?;


    let package = lua.globals().get::<Table>("package")?;
    let preload = package.get::<Table>("preload")?;
    
    preload.set(
        "Koru.StyledText.StyledText",
        lua.create_function(|lua, _:()| {
            let styled_text_module = lua.create_table()?;
            let styled_text_metatable = lua.create_table()?;
            styled_text_metatable.set(
                "__call",
                lua.create_function(|lua, args: mlua::MultiValue| {
                    let (args, vaargs) = args.as_slices();
                    let args = &args[1..];
                    
                    let text = if let Some(mlua::Value::String(string)) = args.first() {
                        string.to_str()?.to_string()
                    } else {
                        todo!("Handle missing string argument")
                    };

                    let text = match vaargs {
                        [] => StyledText::None(text),
                        [mlua::Value::String(fg_color), mlua::Value::String(bg_color), attrs_list @ ..] => {
                            let fg_color = fg_color.to_str()?.to_string();
                            let fg: ColorType = fg_color.as_str().try_into().unwrap();
                            let bg_color = bg_color.to_str()?.to_string();
                            let bg: ColorType = bg_color.as_str().try_into().unwrap();
                            let mut attrs = TextAttribute::empty();
                            for attr in attrs_list {
                                match attr {
                                    mlua::Value::String(attr) => {
                                        let attr = attr.to_str()?.to_string();
                                        match attr.as_str() {
                                            "italic" => attrs |= TextAttribute::Italic,
                                            "bold" => attrs |= TextAttribute::Bold,
                                            "strikethrough" => attrs |= TextAttribute::Strikethrough,
                                            "underline" => attrs |= TextAttribute::Underline,
                                            _ => todo!("raise error over arg values"),
                                        }
                                    }
                                    _ => todo!("raise error over attrs not being a string")
                                }
                            }
                            StyledText::Style {
                                text,
                                fg_color: fg,
                                bg_color: bg,
                                attribute: Arc::new(attrs),
                            }
                        }
                        _ => todo!("raise error over arguments")
                    };

                    lua.create_userdata(text)
                })?
            )?;

            styled_text_module.set_metatable(Some(styled_text_metatable))?;
            Ok(styled_text_module)
        })?
    )?;
    
    preload.set(
        "Koru.StyledText.StyledFile",
        lua.create_function(|lua, _:()| {
            let styled_file_metatable = lua.create_table()?;
            let styled_file_module = lua.create_table()?;

            styled_file_metatable.set(
                "__call",
                lua.create_function(|lua, _: ()| {
                    lua.create_userdata(StyledFile::new())
                })?
            )?;

            styled_file_module.set_metatable(Some(styled_file_metatable))?;
            Ok(styled_file_module)
        })?
    )?;
    
    preload.set(
        "Koru.StyledText.ColorValue",
        lua.create_function(|lua, _:()| {
            let color_value_metatable = lua.create_table()?;
            let color_value_module = lua.create_table()?;
            
            color_value_metatable.set(
                "__call",
                lua.create_function(|lua, args: mlua::MultiValue| {
                    let (args, _) = args.as_slices();
                    let color = match args {
                        [_, mlua::Value::Integer(ansi)] => {
                            ColorValue::Ansi(*ansi as u8)
                        }
                        [_, mlua::Value::Integer(r), mlua::Value::Integer(g), mlua::Value::Integer(b)] => {
                            ColorValue::Rgb {
                                r: *r as u8,
                                g: *g as u8,
                                b: *b as u8,
                            }
                        }
                        _ => todo!("handle error for invalid color value")
                    };
                    lua.create_userdata(color)
                })?
            )?;
            
            color_value_module.set_metatable(Some(color_value_metatable))?;
            Ok(color_value_module)
        })?
    )?;
    
    preload.set(
        "Koru.StyledText.ColorDefinition",
        lua.create_function(|lua, _:()| {
            let color_definition = lua.create_table()?;
            let color_definition_metatable = lua.create_table()?;
            
            color_definition_metatable.set(
                "__call",
                lua.create_function(|lua, (_, def, value): (Table, mlua::String, AnyUserData)| {
                    let value = value.take::<ColorValue>()?;
                    let def = def.to_str()?.to_string();
                    let def = ColorType::try_from(def.as_str()).unwrap();
                    
                    lua.create_userdata(ColorDefinition {
                        color: def,
                        value,
                    })
                })?
            )?;
            
            color_definition.set_metatable(Some(color_definition_metatable))?;
            
            Ok(color_definition)
        })?
    )?;
    
    Ok(exports)
}