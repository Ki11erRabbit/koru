use tuirealm::{AttrValue, Attribute, Component, Event, Frame, MockComponent, State};
use tuirealm::command::{Cmd, CmdResult};
use tuirealm::event::{Key, KeyModifiers};
use tuirealm::ratatui::layout::Rect;
use koru_core::kernel::input::{ControlKey, KeyPress, KeyValue, ModifierKey};
use crate::tuirealm_backend::UiMessage;

pub struct Input;

impl MockComponent for Input {
    fn view(&mut self, _: &mut Frame, _: Rect) {
        
    }

    fn query(&self, _: Attribute) -> Option<AttrValue> {
        None
    }

    fn attr(&mut self, _: Attribute, _: AttrValue) {
        
    }

    fn state(&self) -> State {
        State::None
    }

    fn perform(&mut self, _: Cmd) -> CmdResult {
        CmdResult::None
    }
}

impl Component<UiMessage, UiMessage> for Input {
    fn on(&mut self, ev: Event<UiMessage>) -> Option<UiMessage> {
        match ev {
            Event::User(msg) => {
                Some(msg)
            },
            Event::Keyboard(key_event) => {
                let key = match key_event.code {
                    Key::Esc => KeyValue::ControlKey(ControlKey::Escape),
                    Key::Enter => KeyValue::ControlKey(ControlKey::Enter),
                    Key::Backspace => KeyValue::ControlKey(ControlKey::Backspace),
                    Key::Tab => KeyValue::ControlKey(ControlKey::Tab),
                    Key::Char(' ') => KeyValue::ControlKey(ControlKey::Space),
                    Key::Delete => KeyValue::ControlKey(ControlKey::Delete),
                    Key::Left => KeyValue::ControlKey(ControlKey::Left),
                    Key::Right => KeyValue::ControlKey(ControlKey::Right),
                    Key::Up => KeyValue::ControlKey(ControlKey::Up),
                    Key::Down => KeyValue::ControlKey(ControlKey::Down),
                    Key::Home => KeyValue::ControlKey(ControlKey::Home),
                    Key::End => KeyValue::ControlKey(ControlKey::End),
                    Key::PageUp => KeyValue::ControlKey(ControlKey::PageUp),
                    Key::PageDown => KeyValue::ControlKey(ControlKey::PageDown),
                    Key::Function(1) => KeyValue::ControlKey(ControlKey::F1),
                    Key::Function(2) => KeyValue::ControlKey(ControlKey::F2),
                    Key::Function(3) => KeyValue::ControlKey(ControlKey::F3),
                    Key::Function(4) => KeyValue::ControlKey(ControlKey::F4),
                    Key::Function(5) => KeyValue::ControlKey(ControlKey::F5),
                    Key::Function(6) => KeyValue::ControlKey(ControlKey::F6),
                    Key::Function(7) => KeyValue::ControlKey(ControlKey::F7),
                    Key::Function(8) => KeyValue::ControlKey(ControlKey::F8),
                    Key::Function(9) => KeyValue::ControlKey(ControlKey::F9),
                    Key::Function(10) => KeyValue::ControlKey(ControlKey::F10),
                    Key::Function(11) => KeyValue::ControlKey(ControlKey::F11),
                    Key::Function(12) => KeyValue::ControlKey(ControlKey::F12),
                    Key::Function(13) => KeyValue::ControlKey(ControlKey::F13),
                    Key::Function(14) => KeyValue::ControlKey(ControlKey::F14),
                    Key::Function(15) => KeyValue::ControlKey(ControlKey::F15),
                    Key::Function(16) => KeyValue::ControlKey(ControlKey::F16),
                    Key::Function(17) => KeyValue::ControlKey(ControlKey::F17),
                    Key::Function(18) => KeyValue::ControlKey(ControlKey::F18),
                    Key::Function(19) => KeyValue::ControlKey(ControlKey::F19),
                    Key::Function(20) => KeyValue::ControlKey(ControlKey::F20),
                    Key::Function(21) => KeyValue::ControlKey(ControlKey::F21),
                    Key::Function(22) => KeyValue::ControlKey(ControlKey::F22),
                    Key::Function(23) => KeyValue::ControlKey(ControlKey::F23),
                    Key::Function(24) => KeyValue::ControlKey(ControlKey::F24),
                    Key::Function(25) => KeyValue::ControlKey(ControlKey::F25),
                    Key::Function(26) => KeyValue::ControlKey(ControlKey::F26),
                    Key::Function(27) => KeyValue::ControlKey(ControlKey::F27),
                    Key::Function(28) => KeyValue::ControlKey(ControlKey::F28),
                    Key::Function(29) => KeyValue::ControlKey(ControlKey::F29),
                    Key::Function(30) => KeyValue::ControlKey(ControlKey::F30),
                    Key::Function(31) => KeyValue::ControlKey(ControlKey::F31),
                    Key::Function(32) => KeyValue::ControlKey(ControlKey::F32),
                    Key::Function(33) => KeyValue::ControlKey(ControlKey::F33),
                    Key::Function(34) => KeyValue::ControlKey(ControlKey::F34),
                    Key::Function(35) => KeyValue::ControlKey(ControlKey::F35),
                    Key::Char(c) => KeyValue::CharacterKey(c.to_string().into_boxed_str()),
                    _ => return None,
                };
                let mut modifiers = ModifierKey::empty();
                if key_event.modifiers.contains(KeyModifiers::SHIFT) {
                    modifiers |= ModifierKey::Shift;
                }
                if key_event.modifiers.contains(KeyModifiers::CONTROL) {
                    modifiers |= ModifierKey::Control;
                }
                if key_event.modifiers.contains(KeyModifiers::ALT) {
                    modifiers |= ModifierKey::Alt;
                }
                Some(UiMessage::KeyPress(KeyPress::new(key, modifiers)))
            }
            _ => None,
        }
    }
}