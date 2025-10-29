use std::mem::ManuallyDrop;
use std::sync::LazyLock;
use bitflags::bitflags;
use mlua::{AnyUserData, Lua, Table, UserData, UserDataMethods, Value};
use guile_rs::{Guile, Module, SchemeValue, SmobTag, SmobData, guile_wrong_type_arg, guile_misc_error};
use guile_rs::scheme_object::{SchemeObject, SchemeSmob, SchemeString};
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

pub static STYLED_TEXT_SMOB_TAG: LazyLock<SmobTag<StyledText>> = LazyLock::new(||{
    SmobTag::register("StyledText")
});



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

impl SmobData for StyledText {
    fn heap_size(&self) -> usize {
        size_of::<StyledText>()
    }
}


extern "C" fn styled_text_create(text: SchemeValue, fg: SchemeValue, bg: SchemeValue, rest: SchemeValue) -> SchemeValue {
    let Some(text) = SchemeObject::from(text).cast_string() else {
        guile_wrong_type_arg!("styled-text-create", 1, text);
    };
    let Some(rest) = SchemeObject::from(rest).cast_list() else {
        unreachable!("rest should always be a list");
    };
    let objs = rest.iter().collect::<Vec<_>>();

    let (fg, bg) = match (SchemeObject::from(fg).cast_string(), SchemeObject::from(bg).cast_string()) {
        (Some(fg), Some(bg)) => {
            let Ok(fg) = fg.to_string().as_str().try_into() else {
                guile_wrong_type_arg!("styled-text-create-fg", 2, fg);
            };
            let Ok(bg) = bg.to_string().as_str().try_into() else {
                guile_wrong_type_arg!("styled-text-create-bg", 3, bg);
            };
            (fg, bg)
        }
        _ => {
            let text = StyledText::None(text.to_string());

            return <SchemeSmob<StyledText> as Into<SchemeObject>>::into(STYLED_TEXT_SMOB_TAG.make(text)).into()
        }
    };

    let text = match objs.as_slice() {
        [] => {
            StyledText::Style {
                fg_color: fg,
                bg_color: bg,
                attribute: TextAttribute::empty(),
                text: text.to_string(),
            }
        },
        attrs => {
            let mut attributes = TextAttribute::empty();

            for attr in attrs {
                let Some(attr) = attr.clone().cast_symbol() else {
                    guile_misc_error!("styled-text-create", "attribute is not a symbol", attr.clone());
                };

                match attr.to_string().as_str() {
                    "italic" => attributes |= TextAttribute::Italic,
                    "bold" => attributes |= TextAttribute::Bold,
                    "strikethrough" => attributes |= TextAttribute::Strikethrough,
                    "underline" => attributes |= TextAttribute::Underline,
                    _ => {
                        guile_misc_error!("styled-text-create", "attribute is not one of 'italic, 'bold, 'strikethrough, or 'underline", attr.clone());
                    }
                }
            }
            StyledText::Style {
                fg_color: fg,
                bg_color: bg,
                attribute: attributes,
                text: text.to_string(),
            }
        }
    };

    <SchemeSmob<StyledText> as Into<SchemeObject>>::into(STYLED_TEXT_SMOB_TAG.make(text)).into()
}

pub static STYLED_FILE_SMOB_TAG: LazyLock<SmobTag<StyledFile>> = LazyLock::new(||{
    SmobTag::register("StyledFile")
});


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
                                        attribute: TextAttribute::empty(),
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
                                            attribute: TextAttribute::empty(),
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
                                        attribute,
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
                                        attribute,
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
                                            attribute: TextAttribute::empty(),
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

impl SmobData for StyledFile {
    fn heap_size(&self) -> usize {
        self.lines.capacity() * size_of::<StyledText>()
    }
}

extern "C" fn styled_file_prepend_segment(file: SchemeValue, line: SchemeValue, text: SchemeValue) -> SchemeValue {
    let Some(mut file) = SchemeObject::from(file).cast_smob(STYLED_FILE_SMOB_TAG.clone()) else {
        guile_wrong_type_arg!("styled-file-prepend-segment", 1, file);
    };
    let Some(line) = SchemeObject::from(line).cast_number() else {
        guile_wrong_type_arg!("styled-file-prepend-segment", 2, line);
    };
    let Some(text) = SchemeObject::from(text).cast_smob(STYLED_TEXT_SMOB_TAG.clone()) else {
        guile_wrong_type_arg!("styled-file-prepend-segment", 3, text);
    };
    file.borrow_mut().prepend_segment(line.as_u64() as usize, text.borrow().clone());

    SchemeValue::undefined()
}

extern "C" fn styled_file_append_segment(file: SchemeValue, line: SchemeValue, text: SchemeValue) -> SchemeValue {
    let Some(mut file) = SchemeObject::from(file).cast_smob(STYLED_FILE_SMOB_TAG.clone()) else {
        guile_wrong_type_arg!("styled-file-prepend-segment", 1, file);
    };
    let Some(line) = SchemeObject::from(line).cast_number() else {
        guile_wrong_type_arg!("styled-file-prepend-segment", 2, line);
    };
    let Some(text) = SchemeObject::from(text).cast_smob(STYLED_TEXT_SMOB_TAG.clone()) else {
        guile_wrong_type_arg!("styled-file-prepend-segment", 3, text);
    };
    file.borrow_mut().append_segment(line.as_u64() as usize, text.borrow().clone());

    SchemeValue::undefined()
}

pub fn styled_file_module() {
    Guile::define_fn("styled-file-prepend", 3, 0, false,
        styled_file_prepend_segment as extern "C" fn(SchemeValue, SchemeValue, SchemeValue) -> SchemeValue
    );
    Guile::define_fn("styled-file-append", 3, 0, false,
        styled_file_append_segment as extern "C" fn(SchemeValue, SchemeValue, SchemeValue) -> SchemeValue
    );
    Guile::define_fn("styled-text-create", 1, 2, true,
        styled_text_create as extern "C" fn(SchemeValue, SchemeValue, SchemeValue, SchemeValue) -> SchemeValue
    );

    let mut module = Module::new("styled-text", Box::new(|_: &mut ()| {}));
    module.add_export("styled-file-prepend");
    module.add_export("styled-file-append");
    module.add_export("styled-text-create");
    module.export();
    module.define(&mut ());
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

pub static COLOR_VALUE_SMOB_TAG: LazyLock<SmobTag<ColorValue>> = LazyLock::new(|| {
    SmobTag::register("ColorValue")
});

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
                    
                    let text = if let Some(Value::String(string)) = args.first() {
                        string.to_str()?.to_string()
                    } else {
                        todo!("Handle missing string argument")
                    };

                    let text = match vaargs {
                        [] => StyledText::None(text),
                        [Value::String(fg_color), Value::String(bg_color), attrs_list @ ..] => {
                            let fg_color = fg_color.to_str()?.to_string();
                            let fg: ColorType = fg_color.as_str().try_into().unwrap();
                            let bg_color = bg_color.to_str()?.to_string();
                            let bg: ColorType = bg_color.as_str().try_into().unwrap();
                            let mut attrs = TextAttribute::empty();
                            for attr in attrs_list {
                                match attr {
                                    Value::String(attr) => {
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
                                attribute: attrs,
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
                        [_, Value::Integer(ansi)] => {
                            ColorValue::Ansi(*ansi as u8)
                        }
                        [_, Value::Integer(r), Value::Integer(g), Value::Integer(b)] => {
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