use scheme_rs::records::Record;
use std::sync::{Arc};
use bitflags::bitflags;
use scheme_rs::exceptions::Condition;
use scheme_rs::gc::{OpaqueGcPtr, Trace};
use scheme_rs::lists;
use scheme_rs::records::{rtd, RecordTypeDescriptor, SchemeCompatible};
use scheme_rs::registry::bridge;
use scheme_rs::value::Value;

bitflags! {
    #[derive(Eq, PartialEq, Copy, Clone, Hash, Debug)]
    pub struct ModifierKey: u8 {
        const Shift = 0b0000_0001;
        const Control = 0b0000_0010;
        const Alt = 0b0000_0100;
        const Meta = 0b0000_1000;
    }
}

unsafe impl Trace for ModifierKey {
    unsafe fn visit_children(&self, _: &mut dyn FnMut(OpaqueGcPtr)) {
        
    }
}

impl SchemeCompatible for ModifierKey {
    fn rtd() -> Arc<RecordTypeDescriptor>
    where
        Self: Sized
    {
        rtd!(name: "&ModifierKey")
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

#[derive(Eq, PartialEq, Copy, Clone, Hash, Debug, Trace)]
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

#[derive(Eq, PartialEq, Copy, Clone, Hash, Debug, Trace)]
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


#[derive(Eq, PartialEq, Copy, Clone, Hash, Debug, Trace)]
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
            c if c.chars().count() == 1 => {
                let Some(char) = c.chars().next() else {
                    unreachable!("We have already asserted this");
                };
                KeyValue::CharacterKey(char)
            }
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

impl SchemeCompatible for KeyPress {
    fn rtd() -> Arc<RecordTypeDescriptor>
    where
        Self: Sized
    {
        rtd!(name: "&KeyPress")
    }
}

impl std::fmt::Display for KeyPress {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{}", self.modifiers)?;

        if !self.modifiers.is_empty() {
            write!(f, "-")?;
        }

        write!(f, "{}", self.key)
    }
}

#[bridge(name = "string->keypress", lib = "(key-press)")]
pub fn string_to_keypress(string: &Value) -> Result<Vec<Value>, Condition> {
    let string: String = string.clone().try_into()?;
    let Some(keypress) = KeyPress::from_string(&string) else {
        return Err(Condition::error(string.to_string()));
    };

    Ok(vec![Value::from(Record::from_rust_type(keypress))])
}

#[bridge(name = "string->keysequence", lib = "(key-press)")]
pub fn string_to_key_list(string: &Value) -> Result<Vec<Value>, Condition> {
    let string: String = string.clone().try_into()?;
    let vec = string.split_whitespace()
        .map(|s| {
            KeyPress::from_string(s).map(|kp| Value::from(Record::from_rust_type(kp)))
        })
        .collect::<Option<Vec<Value>>>();
    let Some(vec) = vec else {
        return Err(Condition::error(String::from("Invalid key sequence.")));
    };
    
    let value = lists::slice_to_list(&vec);

    Ok(vec![value])
}