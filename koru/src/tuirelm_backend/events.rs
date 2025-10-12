use std::io;
use std::sync::mpsc::{Receiver, TryRecvError};
use std::time::Duration;
use tuirealm::listener::{ListenerResult, Poll};
use tuirealm::ListenerError;
use tuirealm::ratatui::crossterm::event;
use tuirealm::ratatui::crossterm::event::{poll, Event, KeyCode, KeyEventKind, KeyModifiers};
use koru_core::kernel::broker::BrokerClient;
use koru_core::kernel::input::{ControlKey, KeyPress, KeyValue, ModifierKey};
use crate::common::UiMessage;
use crate::tuirelm_backend::App;

pub fn handle_event(state: &mut App) -> io::Result<Option<UiMessage>> {
    if poll(Duration::from_millis(100))? {
        match event::read()? {
            Event::Key(key_event) => {
                match key_event.kind {
                    KeyEventKind::Release => {
                        return Ok(None);
                    }
                    _ => {}
                }
                let key = match key_event.code {
                    KeyCode::Enter => KeyValue::ControlKey(ControlKey::Enter),
                    KeyCode::Backspace => KeyValue::ControlKey(ControlKey::Backspace),
                    KeyCode::Tab => KeyValue::ControlKey(ControlKey::Tab),
                    KeyCode::Char(' ') => KeyValue::ControlKey(ControlKey::Space),
                    KeyCode::Delete => KeyValue::ControlKey(ControlKey::Delete),
                    KeyCode::Left => KeyValue::ControlKey(ControlKey::Left),
                    KeyCode::Right => KeyValue::ControlKey(ControlKey::Right),
                    KeyCode::Up => KeyValue::ControlKey(ControlKey::Up),
                    KeyCode::Down => KeyValue::ControlKey(ControlKey::Down),
                    KeyCode::Home => KeyValue::ControlKey(ControlKey::Home),
                    KeyCode::End => KeyValue::ControlKey(ControlKey::End),
                    KeyCode::PageUp => KeyValue::ControlKey(ControlKey::PageUp),
                    KeyCode::PageDown => KeyValue::ControlKey(ControlKey::PageDown),
                    KeyCode::F(1) => KeyValue::ControlKey(ControlKey::F1),
                    KeyCode::F(2) => KeyValue::ControlKey(ControlKey::F2),
                    KeyCode::F(3) => KeyValue::ControlKey(ControlKey::F3),
                    KeyCode::F(4) => KeyValue::ControlKey(ControlKey::F4),
                    KeyCode::F(5) => KeyValue::ControlKey(ControlKey::F5),
                    KeyCode::F(6) => KeyValue::ControlKey(ControlKey::F6),
                    KeyCode::F(7) => KeyValue::ControlKey(ControlKey::F7),
                    KeyCode::F(8) => KeyValue::ControlKey(ControlKey::F8),
                    KeyCode::F(9) => KeyValue::ControlKey(ControlKey::F9),
                    KeyCode::F(10) => KeyValue::ControlKey(ControlKey::F10),
                    KeyCode::F(11) => KeyValue::ControlKey(ControlKey::F11),
                    KeyCode::F(12) => KeyValue::ControlKey(ControlKey::F12),
                    KeyCode::F(13) => KeyValue::ControlKey(ControlKey::F13),
                    KeyCode::F(14) => KeyValue::ControlKey(ControlKey::F14),
                    KeyCode::F(15) => KeyValue::ControlKey(ControlKey::F15),
                    KeyCode::F(16) => KeyValue::ControlKey(ControlKey::F16),
                    KeyCode::F(17) => KeyValue::ControlKey(ControlKey::F17),
                    KeyCode::F(18) => KeyValue::ControlKey(ControlKey::F18),
                    KeyCode::F(19) => KeyValue::ControlKey(ControlKey::F19),
                    KeyCode::F(20) => KeyValue::ControlKey(ControlKey::F20),
                    KeyCode::F(21) => KeyValue::ControlKey(ControlKey::F21),
                    KeyCode::F(22) => KeyValue::ControlKey(ControlKey::F22),
                    KeyCode::F(23) => KeyValue::ControlKey(ControlKey::F23),
                    KeyCode::F(24) => KeyValue::ControlKey(ControlKey::F24),
                    KeyCode::F(25) => KeyValue::ControlKey(ControlKey::F25),
                    KeyCode::F(26) => KeyValue::ControlKey(ControlKey::F26),
                    KeyCode::F(27) => KeyValue::ControlKey(ControlKey::F27),
                    KeyCode::F(28) => KeyValue::ControlKey(ControlKey::F28),
                    KeyCode::F(29) => KeyValue::ControlKey(ControlKey::F29),
                    KeyCode::F(30) => KeyValue::ControlKey(ControlKey::F30),
                    KeyCode::F(31) => KeyValue::ControlKey(ControlKey::F31),
                    KeyCode::F(32) => KeyValue::ControlKey(ControlKey::F32),
                    KeyCode::F(33) => KeyValue::ControlKey(ControlKey::F33),
                    KeyCode::F(34) => KeyValue::ControlKey(ControlKey::F34),
                    KeyCode::F(35) => KeyValue::ControlKey(ControlKey::F35),
                    KeyCode::Char(c) => KeyValue::CharacterKey(c),
                    _ => return Ok(None),
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
                Ok(Some(UiMessage::KeyPress(KeyPress::new(key, modifiers))))
            }
            Event::Resize(..) => {
                state.redraw = true;
                Ok(None)
            }
            _ => Ok(None),
        }
    } else {
        Ok(None)
    }
}


pub struct BrokerPort {
    receiver: Receiver<UiMessage>,
}

impl BrokerPort {
    pub fn new(broker_client: &mut BrokerClient) -> BrokerPort {
        let mut client = broker_client.clone();
        std::mem::swap(broker_client, &mut client);
        let (sender, receiver) = std::sync::mpsc::channel();
        
        koru_core::spawn_task(async move {
            loop {
                match client.recv().await {
                    Some(msg) => {
                        sender.send(UiMessage::BrokerMessage(msg)).unwrap();
                    }
                    None => break,
                }
            }
        });
        
        //let value = receiver.recv().unwrap();
        //println!("BrokerPort Sent: {:?}", value);
        
        BrokerPort { receiver }
    }
}

impl Poll<UiMessage> for BrokerPort {
    fn poll(&mut self) -> ListenerResult<Option<tuirealm::Event<UiMessage>>> {
        //println!("BrokerPort Poll");
        match self.receiver.try_recv() {
            Ok(msg) => {
                Ok(Some(tuirealm::Event::User(msg)))
            }
            Err(TryRecvError::Disconnected) => {
                Err(ListenerError::ListenerDied)
            },
            _ => Ok(None),
        }
    }
}