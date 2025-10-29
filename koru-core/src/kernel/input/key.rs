use std::fmt::format;
use std::ops::BitAnd;
use std::sync::{Arc, LazyLock};
use bitflags::bitflags;
use mlua::{Lua, UserData, UserDataMethods, Value};
use guile_rs::scheme_object::{SchemeObject, SchemeSmob};
use guile_rs::{guile_misc_error, guile_wrong_type_arg, Guile, Module, SchemeValue, SmobData, SmobTag};

bitflags! {
    #[derive(Eq, PartialEq, Copy, Clone, Hash, Debug)]
    pub struct ModifierKey: u8 {
        const Shift = 0b0000_0001;
        const Control = 0b0000_0010;
        const Alt = 0b0000_0100;
        const Meta = 0b0000_1000;
    }
}

impl std::fmt::Display for ModifierKey {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut written_once = false;
        if *self & ModifierKey::Meta == ModifierKey::Meta {
            written_once = true;
            write!(f, "M")?;
        }
        if *self & ModifierKey::Shift == ModifierKey::Shift {
            if written_once {
                write!(f, "-")?;
            }
            written_once = true;
            write!(f, "S")?;
        }
        if *self & ModifierKey::Control == ModifierKey::Control {
            if written_once {
                write!(f, "-")?;
            }
            written_once = true;
            write!(f, "C")?;
        }
        if *self & ModifierKey::Alt == ModifierKey::Alt {
            if written_once {
                write!(f, "-")?;
            }
            write!(f, "A")?;
        }
        Ok(())
    }
}

#[derive(Eq, PartialEq, Copy, Clone, Hash, Debug)]
pub enum KeyValue {
    CharacterKey(char),
    ControlKey(ControlKey),
}

impl std::fmt::Display for KeyValue {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            KeyValue::CharacterKey('-') => write!(f, "DASH"),
            KeyValue::CharacterKey(c) => write!(f, "{}", c),
            KeyValue::ControlKey(c) => write!(f, "{}", c),
        }
    }
}

#[derive(Eq, PartialEq, Copy, Clone, Hash, Debug)]
pub enum ControlKey {
    Enter,
    Tab,
    Space,
    Escape,
    Backspace,
    Delete,
    Left,
    Right,
    Up,
    Down,
    PageUp,
    PageDown,
    Home,
    End,
    F1,
    F2,
    F3,
    F4,
    F5,
    F6,
    F7,
    F8,
    F9,
    F10,
    F11,
    F12,
    F13,
    F14,
    F15,
    F16,
    F17,
    F18,
    F19,
    F20,
    F21,
    F22,
    F23,
    F24,
    F25,
    F26,
    F27,
    F28,
    F29,
    F30,
    F31,
    F32,
    F33,
    F34,
    F35,
}

impl std::fmt::Display for ControlKey {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ControlKey::Enter => write!(f, "ENTER"),
            ControlKey::Tab => write!(f, "TAB"),
            ControlKey::Space => write!(f, "SPC"),
            ControlKey::Escape => write!(f, "ESC"),
            ControlKey::Backspace => write!(f, "BS"),
            ControlKey::Delete => write!(f, "DEL"),
            ControlKey::Left => write!(f, "LEFT"),
            ControlKey::Right => write!(f, "RIGHT"),
            ControlKey::Up => write!(f, "UP"),
            ControlKey::Down => write!(f, "DOWN"),
            ControlKey::PageUp => write!(f, "PAGEUP"),
            ControlKey::PageDown => write!(f, "PAGEDOWN"),
            ControlKey::Home => write!(f, "HOME"),
            ControlKey::End => write!(f, "END"),
            ControlKey::F1 => write!(f, "F1"),
            ControlKey::F2 => write!(f, "F2"),
            ControlKey::F3 => write!(f, "F3"),
            ControlKey::F4 => write!(f, "F4"),
            ControlKey::F5 => write!(f, "F5"),
            ControlKey::F6 => write!(f, "F6"),
            ControlKey::F7 => write!(f, "F7"),
            ControlKey::F8 => write!(f, "F8"),
            ControlKey::F9 => write!(f, "F9"),
            ControlKey::F10 => write!(f, "F10"),
            ControlKey::F11 => write!(f, "F11"),
            ControlKey::F12 => write!(f, "F12"),
            ControlKey::F13 => write!(f, "F13"),
            ControlKey::F14 => write!(f, "F14"),
            ControlKey::F15 => write!(f, "F15"),
            ControlKey::F16 => write!(f, "F16"),
            ControlKey::F17 => write!(f, "F17"),
            ControlKey::F18 => write!(f, "F18"),
            ControlKey::F19 => write!(f, "F19"),
            ControlKey::F20 => write!(f, "F20"),
            ControlKey::F21 => write!(f, "F21"),
            ControlKey::F22 => write!(f, "F22"),
            ControlKey::F23 => write!(f, "F23"),
            ControlKey::F24 => write!(f, "F24"),
            ControlKey::F25 => write!(f, "F25"),
            ControlKey::F26 => write!(f, "F26"),
            ControlKey::F27 => write!(f, "F27"),
            ControlKey::F28 => write!(f, "F28"),
            ControlKey::F29 => write!(f, "F29"),
            ControlKey::F30 => write!(f, "F30"),
            ControlKey::F31 => write!(f, "F31"),
            ControlKey::F32 => write!(f, "F32"),
            ControlKey::F33 => write!(f, "F33"),
            ControlKey::F34 => write!(f, "F34"),
            ControlKey::F35 => write!(f, "F35"),
        }
    }
}

pub static KEY_PRESS_SMOB_TAG: LazyLock<SmobTag<KeyPress>> = LazyLock::new(|| {
    SmobTag::register("KeyPress")
});

#[derive(Eq, PartialEq, Clone, Hash, Debug)]
pub struct KeyPress {
    pub key: KeyValue,
    pub modifiers: ModifierKey,
}

impl KeyPress {
    pub fn new(key: KeyValue, modifiers: ModifierKey) -> KeyPress {
        KeyPress { key, modifiers }
    }

    pub fn is_shift_pressed(&self) -> bool {
        if self.modifiers & ModifierKey::Shift == ModifierKey::Shift {
            return true;
        }

        match self.key {
            KeyValue::CharacterKey('!') => true,
            KeyValue::CharacterKey('@') => true,
            KeyValue::CharacterKey('#') => true,
            KeyValue::CharacterKey('$') => true,
            KeyValue::CharacterKey('%') => true,
            KeyValue::CharacterKey('^') => true,
            KeyValue::CharacterKey('&') => true,
            KeyValue::CharacterKey('*') => true,
            KeyValue::CharacterKey('(') => true,
            KeyValue::CharacterKey(')') => true,
            KeyValue::CharacterKey('{') => true,
            KeyValue::CharacterKey('}') => true,
            KeyValue::CharacterKey('~') => true,
            KeyValue::CharacterKey('?') => true,
            KeyValue::CharacterKey('"') => true,
            KeyValue::CharacterKey('|') => true,
            KeyValue::CharacterKey('_') => true,
            KeyValue::CharacterKey(':') => true,
            KeyValue::CharacterKey('+') => true,
            KeyValue::CharacterKey('<') => true,
            KeyValue::CharacterKey('>') => true,
            KeyValue::CharacterKey(key) => key.is_uppercase(),
            _ => false,
        }
    }

    pub fn is_control_pressed(&self) -> bool {
        if self.modifiers & ModifierKey::Control == ModifierKey::Control {
            true
        } else {
            false
        }
    }

    pub fn is_alt_pressed(&self) -> bool {
        if self.modifiers & ModifierKey::Alt == ModifierKey::Alt {
            true
        } else {
            false
        }
    }

    pub fn is_meta_pressed(&self) -> bool {
        if self.modifiers & ModifierKey::Meta == ModifierKey::Meta {
            true
        } else {
            false
        }
    }

    pub fn key_string(&self) -> String {
        format!("{}", self.key)
    }

    fn match_key_string(key_string: &str) -> Option<KeyValue> {
        let key = match key_string {
            "SPC" => KeyValue::ControlKey(ControlKey::Space),
            "TAB" => KeyValue::ControlKey(ControlKey::Tab),
            "ENTER" => KeyValue::ControlKey(ControlKey::Enter),
            "DASH" => KeyValue::CharacterKey('-'),
            "ESC" => KeyValue::ControlKey(ControlKey::Escape),
            "BS" => KeyValue::ControlKey(ControlKey::Backspace),
            "DEL" => KeyValue::ControlKey(ControlKey::Delete),
            "UP" => KeyValue::ControlKey(ControlKey::Up),
            "DOWN" => KeyValue::ControlKey(ControlKey::Down),
            "LEFT" => KeyValue::ControlKey(ControlKey::Left),
            "RIGHT" => KeyValue::ControlKey(ControlKey::Right),
            "HOME" => KeyValue::ControlKey(ControlKey::Home),
            "END" => KeyValue::ControlKey(ControlKey::End),
            "PAGEUP" => KeyValue::ControlKey(ControlKey::PageUp),
            "PAGEDOWN" => KeyValue::ControlKey(ControlKey::PageDown),
            "F1" => KeyValue::ControlKey(ControlKey::F1),
            "F2" => KeyValue::ControlKey(ControlKey::F2),
            "F3" => KeyValue::ControlKey(ControlKey::F3),
            "F4" => KeyValue::ControlKey(ControlKey::F4),
            "F5" => KeyValue::ControlKey(ControlKey::F5),
            "F6" => KeyValue::ControlKey(ControlKey::F6),
            "F7" => KeyValue::ControlKey(ControlKey::F7),
            "F8" => KeyValue::ControlKey(ControlKey::F8),
            "F9" => KeyValue::ControlKey(ControlKey::F9),
            "F10" => KeyValue::ControlKey(ControlKey::F10),
            "F11" => KeyValue::ControlKey(ControlKey::F11),
            "F12" => KeyValue::ControlKey(ControlKey::F12),
            "F13" => KeyValue::ControlKey(ControlKey::F13),
            "F14" => KeyValue::ControlKey(ControlKey::F14),
            "F15" => KeyValue::ControlKey(ControlKey::F15),
            "F16" => KeyValue::ControlKey(ControlKey::F16),
            "F17" => KeyValue::ControlKey(ControlKey::F17),
            "F18" => KeyValue::ControlKey(ControlKey::F18),
            "F19" => KeyValue::ControlKey(ControlKey::F19),
            "F20" => KeyValue::ControlKey(ControlKey::F20),
            "F21" => KeyValue::ControlKey(ControlKey::F21),
            "F22" => KeyValue::ControlKey(ControlKey::F22),
            "F23" => KeyValue::ControlKey(ControlKey::F23),
            "F24" => KeyValue::ControlKey(ControlKey::F24),
            "F25" => KeyValue::ControlKey(ControlKey::F25),
            "F26" => KeyValue::ControlKey(ControlKey::F26),
            "F27" => KeyValue::ControlKey(ControlKey::F27),
            "F28" => KeyValue::ControlKey(ControlKey::F28),
            "F29" => KeyValue::ControlKey(ControlKey::F29),
            "F30" => KeyValue::ControlKey(ControlKey::F30),
            "F31" => KeyValue::ControlKey(ControlKey::F31),
            "F32" => KeyValue::ControlKey(ControlKey::F32),
            "F33" => KeyValue::ControlKey(ControlKey::F33),
            "F34" => KeyValue::ControlKey(ControlKey::F34),
            "F35" => KeyValue::ControlKey(ControlKey::F35),
            _ => return None,
        };
        Some(key)
    }

    pub fn from_string(string: &str) -> Option<KeyPress> {
        let strings = string.split('-').collect::<Vec<&str>>();
        let mut modifiers = ModifierKey::empty();
        for (i, string) in strings.iter().enumerate() {
            if i < strings.len() - 1 {
                match *string {
                    "S" => modifiers |= ModifierKey::Shift,
                    "C" => modifiers |= ModifierKey::Control,
                    "A" => modifiers |= ModifierKey::Alt,
                    "M" => modifiers |= ModifierKey::Meta,
                    _ => return None,
                }
            } else {
                let key = Self::match_key_string(string)?;
                return Some(KeyPress::new(key, modifiers));
            }
        }
        None
    }
}


impl SmobData for KeyPress {
    fn print(&self) -> String {
        format!("{}", self)
    }

    fn heap_size(&self) -> usize {
        0
    }

    fn eq(&self, other: SchemeObject) -> bool {
        let Some(other) = other.cast_smob(KEY_PRESS_SMOB_TAG.clone()) else {
            return false
        };
        *self == *other.borrow()
    }
}

impl std::fmt::Display for KeyPress {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{}", self.modifiers)?;
        
        write!(f, "{}", self.key)
    }
}

extern "C" fn string_to_keypress(string: SchemeValue) -> SchemeValue {
    let Some(string) = SchemeObject::from(string).cast_string() else {
        guile_wrong_type_arg!("string->keypress", 1, string);
    };
    let Some(press) = KeyPress::from_string(&string.to_string()) else {
        guile_misc_error!("string->keypress", "invalid string for string->keypress");
    };
    let smob = KEY_PRESS_SMOB_TAG.make(press);
    
    <SchemeSmob<KeyPress> as Into<SchemeObject>>::into(smob).into()
}

pub fn guile_key_module() {
    Guile::define_fn("string->keypress", 1, 0, false,
                     string_to_keypress as extern "C" fn(SchemeValue) -> SchemeValue
    );
    let mut module: Module<()> = Module::new_default("key-press");
    module.add_export("string->keypress");
    module.export();
    module.define_default();
}

impl UserData for KeyPress {
    fn add_methods<M: UserDataMethods<Self>>(methods: &mut M) {
        /*methods.add_function(
            "is_shift_pressed",
            |_, this: AnyUserData, _:()| {
                let this = this.borrow::<KeyPress>()?;
                Ok(this.is_shift_pressed())
            }
        );
        methods.add_function(
            "is_control_pressed",
            |_, this: AnyUserData, _:()| {
                let this = this.borrow::<KeyPress>()?;
                Ok(this.is_control_pressed())
            }
        );
        methods.add_function(
            "is_alt_pressed",
            |_, this: AnyUserData, _:()| {
                let this = this.borrow::<KeyPress>()?;
                Ok(this.is_alt_pressed())
            }
        );
        methods.add_function(
            "key_string",
            |_, this: AnyUserData, _:()| {
                let this = this.borrow::<KeyPress>()?;
                Ok(this.key_string())
            }
        );*/
    }
}


pub fn key_module(lua: &Lua) -> mlua::Result<mlua::Table> {
    let exports = lua.create_table()?;

    let meta = lua.create_table()?;

    meta.set(
        "__call",
        lua.create_function(|lua, string: String| {
            let key_press = KeyPress::from_string(&string)
                .ok_or(mlua::Error::external(String::from("invalid key string")))?;
            lua.create_userdata(key_press)
        })?
    )?;
    exports.set_metatable(Some(meta))?;
    
    exports.set(
        "create_seq",
        lua.create_function(|lua, mut args: mlua::MultiValue| {
            let (_, vaargs) = args.as_mut_slices();
            let table = lua.create_table()?;
            for (i, value) in vaargs.iter_mut().enumerate() {
                match value {
                    Value::String(string) => {
                        let key_string = string.to_str()?.to_string();
                        let key = KeyPress::from_string(&key_string)
                            .ok_or(mlua::Error::external(String::from("invalid key string")))?;
                        table.set(
                            i + 1,
                            lua.create_userdata(key)?
                        )?;
                    }
                    Value::UserData(data) => {
                        let data = data.take::<KeyPress>()?;
                        table.set(
                            i + 1,
                            lua.create_userdata(data)?
                        )?
                    }
                    _ => {
                        return Err(mlua::Error::BadArgument {
                            to: None,
                            pos: i,
                            name: None,
                            cause: Arc::new(mlua::Error::external(String::from("value must be string or keypress"))),
                        })
                    }
                }
            }
            Ok(table)
        })?,
    )?;

    Ok(exports)
}