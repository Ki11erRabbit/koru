use std::num::NonZeroU8;
use mlua::{AnyUserData, Lua, UserData, UserDataMethods};

#[derive(Eq, PartialEq, Copy, Clone, Hash)]
pub enum ModifierKey {
    Shift,
    Control,
    Alt,
}

impl std::fmt::Display for ModifierKey {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ModifierKey::Shift => write!(f, "S"),
            ModifierKey::Control => write!(f, "C"),
            ModifierKey::Alt => write!(f, "A"),
        }
    }
}

#[derive(Eq, PartialEq, Copy, Clone, Hash)]
pub enum Key {
    CharacterKey(char),
    ControlKey(ControlKey),
}

impl std::fmt::Display for Key {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Key::CharacterKey(' ') => write!(f, "SPC"),
            Key::CharacterKey('\t') => write!(f, "TAB"),
            Key::CharacterKey('\n') => write!(f, "LF"),
            Key::CharacterKey('\r') => write!(f, "LF"),
            Key::CharacterKey('-') => write!(f, "DASH"),
            Key::CharacterKey(c) => write!(f, "{}", c),
            Key::ControlKey(c) => write!(f, "{}", c),
        }
    }
}

#[derive(Eq, PartialEq, Copy, Clone, Hash)]
pub enum ControlKey {
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
    F(NonZeroU8),
}

impl std::fmt::Display for ControlKey {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
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
            ControlKey::F(n) => write!(f, "F{}", n.get()),
        }
    }
}

#[derive(Eq, PartialEq, Copy, Clone, Hash)]
pub struct KeyPress {
    pub key: Key,
    pub modifier: Option<ModifierKey>,
}

impl KeyPress {
    pub fn new(key: Key, modifier: Option<ModifierKey>) -> KeyPress {
        KeyPress { key, modifier }
    }

    pub fn is_shift_pressed(&self) -> bool {
        match self.modifier {
            Some(ModifierKey::Shift) => return true,
            _ => {}
        }

        match self.key {
            Key::CharacterKey('!') => true,
            Key::CharacterKey('@') => true,
            Key::CharacterKey('#') => true,
            Key::CharacterKey('$') => true,
            Key::CharacterKey('%') => true,
            Key::CharacterKey('^') => true,
            Key::CharacterKey('&') => true,
            Key::CharacterKey('*') => true,
            Key::CharacterKey('(') => true,
            Key::CharacterKey(')') => true,
            Key::CharacterKey('{') => true,
            Key::CharacterKey('}') => true,
            Key::CharacterKey('~') => true,
            Key::CharacterKey('?') => true,
            Key::CharacterKey('"') => true,
            Key::CharacterKey('|') => true,
            Key::CharacterKey('_') => true,
            Key::CharacterKey(':') => true,
            Key::CharacterKey('+') => true,
            Key::CharacterKey('<') => true,
            Key::CharacterKey('>') => true,
            Key::CharacterKey(key) => key.is_uppercase(),
            _ => false,
        }
    }

    pub fn is_control_pressed(&self) -> bool {
        match self.modifier {
            Some(ModifierKey::Control) => true,
            _ => false,
        }
    }

    pub fn is_alt_pressed(&self) -> bool {
        match self.modifier {
            Some(ModifierKey::Alt) => true,
            _ => false,
        }
    }

    pub fn key_string(&self) -> String {
        format!("{}", self.key)
    }

    fn match_key_string(key_string: &str) -> Option<Key> {
        let key = match key_string {
            "SPC" => Key::CharacterKey(' '),
            "TAB" => Key::CharacterKey('\t'),
            "LF" => Key::CharacterKey('\n'),
            "DASH" => Key::CharacterKey('-'),
            "ESC" => Key::ControlKey(ControlKey::Escape),
            "BS" => Key::ControlKey(ControlKey::Backspace),
            "DEL" => Key::ControlKey(ControlKey::Delete),
            "UP" => Key::ControlKey(ControlKey::Up),
            "DOWN" => Key::ControlKey(ControlKey::Down),
            "LEFT" => Key::ControlKey(ControlKey::Left),
            "RIGHT" => Key::ControlKey(ControlKey::Right),
            "HOME" => Key::ControlKey(ControlKey::Home),
            "END" => Key::ControlKey(ControlKey::End),
            "PAGEUP" => Key::ControlKey(ControlKey::PageUp),
            "PAGEDOWN" => Key::ControlKey(ControlKey::PageDown),
            "F1" => Key::ControlKey(ControlKey::F(NonZeroU8::new(1).unwrap())),
            "F2" => Key::ControlKey(ControlKey::F(NonZeroU8::new(1).unwrap())),
            "F3" => Key::ControlKey(ControlKey::F(NonZeroU8::new(1).unwrap())),
            "F4" => Key::ControlKey(ControlKey::F(NonZeroU8::new(1).unwrap())),
            "F5" => Key::ControlKey(ControlKey::F(NonZeroU8::new(1).unwrap())),
            "F6" => Key::ControlKey(ControlKey::F(NonZeroU8::new(1).unwrap())),
            "F7" => Key::ControlKey(ControlKey::F(NonZeroU8::new(1).unwrap())),
            "F8" => Key::ControlKey(ControlKey::F(NonZeroU8::new(1).unwrap())),
            "F9" => Key::ControlKey(ControlKey::F(NonZeroU8::new(1).unwrap())),
            "F10" => Key::ControlKey(ControlKey::F(NonZeroU8::new(1).unwrap())),
            "F11" => Key::ControlKey(ControlKey::F(NonZeroU8::new(1).unwrap())),
            "F12" => Key::ControlKey(ControlKey::F(NonZeroU8::new(1).unwrap())),
            "F13" => Key::ControlKey(ControlKey::F(NonZeroU8::new(1).unwrap())),
            "F14" => Key::ControlKey(ControlKey::F(NonZeroU8::new(1).unwrap())),
            "F15" => Key::ControlKey(ControlKey::F(NonZeroU8::new(1).unwrap())),
            "F16" => Key::ControlKey(ControlKey::F(NonZeroU8::new(1).unwrap())),
            "F17" => Key::ControlKey(ControlKey::F(NonZeroU8::new(1).unwrap())),
            "F18" => Key::ControlKey(ControlKey::F(NonZeroU8::new(1).unwrap())),
            "F19" => Key::ControlKey(ControlKey::F(NonZeroU8::new(1).unwrap())),
            "F20" => Key::ControlKey(ControlKey::F(NonZeroU8::new(1).unwrap())),
            "F21" => Key::ControlKey(ControlKey::F(NonZeroU8::new(1).unwrap())),
            "F22" => Key::ControlKey(ControlKey::F(NonZeroU8::new(1).unwrap())),
            "F23" => Key::ControlKey(ControlKey::F(NonZeroU8::new(1).unwrap())),
            "F24" => Key::ControlKey(ControlKey::F(NonZeroU8::new(1).unwrap())),
            _ => return None,
        };
        Some(key)
    }

    pub fn from_string(string: &str) -> Option<KeyPress> {
        let strings = string.split('-').collect::<Vec<&str>>();
        match strings.as_slice() {
            ["S", key] =>  {
                let key = Self::match_key_string(*key)?;

                Some(KeyPress::new(key, Some(ModifierKey::Shift)))
            }
            ["C", key] => {
                let key = Self::match_key_string(*key)?;
                Some(KeyPress::new(key, Some(ModifierKey::Control)))
            }
            ["A", key] => {
                let key = Self::match_key_string(*key)?;
                Some(KeyPress::new(key, Some(ModifierKey::Alt)))
            }
            [key] => {
                let key = Self::match_key_string(*key)?;
                Some(KeyPress::new(key, None))
            }
            _ => None,
        }

    }
}

impl std::fmt::Display for KeyPress {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match &self.modifier {
            Some(modifier) => {
                modifier.fmt(f)?;
                write!(f, "-")?;
            },
            None => {
                write!(f, "")?;
            },
        }
        write!(f, "{}", self.key)
    }
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

    Ok(exports)
}