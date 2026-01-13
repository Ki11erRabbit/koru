mod buffer;
mod keymap;

use std::ops::DerefMut;
use std::ops::Deref;
pub use buffer::*;

use std::collections::HashMap;
use std::sync::{Arc, LazyLock};
use log::{error, info};
use scheme_rs::exceptions::Condition;
use scheme_rs::gc::Gc;
use scheme_rs::lists;
use scheme_rs::proc::Procedure;
use scheme_rs::records::Record;
use scheme_rs::registry::bridge;
use scheme_rs::symbols::Symbol;
use scheme_rs::value::{UnpackedValue, Value};
use tokio::sync::{RwLock};
use keypress_localize::KeyboardRegion;
use crate::kernel::broker::{BackendMessage, BrokerClient, MessageKind};
use crate::kernel::buffer::{BufferHandle};
use crate::kernel::input::{KeyBuffer, KeyPress, KeyValue};
use crate::kernel::scheme_api::command::Command;
use crate::kernel::scheme_api::major_mode::MajorMode;
use crate::kernel::scheme_api::minor_mode::MinorMode;
use crate::kernel::scheme_api::session::keymap::SchemeKeyMap;
use crate::keymap::KeyMap;

pub struct Hooks {
    hooks: HashMap<Symbol, HashMap<Symbol, Procedure>>,
}

impl Hooks {
    fn new() -> Self {
        Hooks {
            hooks: HashMap::new(),
        }
    }

    pub fn add_new_hook_kind(&mut self, name: Symbol) {
        self.hooks.insert(name, HashMap::new());
    }

    pub fn remove_hook_kind(&mut self, name: &Symbol) {
        self.hooks.remove(name);
    }

    pub fn add_new_hook(&mut self, hook_name: Symbol, name: Symbol, procedure: Procedure) {
        let Some(hooks) = self.hooks.get_mut(&hook_name) else {
            panic!("Unknown hook {}", hook_name);
        };
        hooks.insert(name, procedure);
    }

    pub fn remove_hook(&mut self, hook_name: &Symbol, name: &Symbol) {
        if let Some(hooks) = self.hooks.get_mut(hook_name) {
            hooks.remove(name);
        }
    }

    pub async fn execute_hook(&self, hook_name: &Symbol, args: &[Value]) -> Result<(), Condition> {
        let Some(hooks) = self.hooks.get(hook_name) else {
            panic!("Unknown hook {}", hook_name);
        };
        for (_, procedure) in hooks.iter() {
            procedure.call(args).await?;
        }
        Ok(())
    }
}


struct KeyPressCommandResult {
    pub flush: bool,
    pub found: bool,
}

impl KeyPressCommandResult {
    fn new(flush: bool, found: bool) -> Self {
        KeyPressCommandResult { flush, found }
    }
}

pub struct CommandBar {
    buffer: String,
    cursor: usize,
}

impl CommandBar {
    pub fn new() -> Self {
        Self {
            buffer: String::new(),
            cursor: 0,
        }
    }

    pub fn cursor_left(&mut self) {
        self.cursor = self.cursor.saturating_sub(1);
    }
    pub fn cursor_right(&mut self) {
        if self.cursor != self.buffer.chars().count() {
            self.cursor = self.cursor.saturating_add(1);
        }
    }

    pub fn delete_backward(&mut self) {
        let chars = self.buffer.chars()
            .take(self.cursor - 1)
            .chain(self.buffer.chars().skip(self.cursor))
            .collect::<String>();
        self.buffer = chars;
        self.cursor -= 1;
    }

    pub fn delete_forward(&mut self) {
        let chars = self.buffer.chars()
        .take(self.cursor)
        .chain(self.buffer.chars().skip(self.cursor + 1))
        .collect::<String>();
        self.buffer = chars;
    }

    pub fn take(&mut self) -> String {
        self.cursor = 0;
        std::mem::take(&mut self.buffer)
    }

    pub fn get(&self) -> String {
        self.buffer.clone()
    }

    pub fn insert(&mut self, value: &str) {
        let index = self.buffer.chars()
            .take(self.cursor)
            .map(char::len_utf8)
            .sum();
        let char_count = value.chars().count();
        self.buffer.insert_str(index, value);
        self.cursor += char_count;
    }
}

pub struct SessionState {
    buffers: Arc<RwLock<HashMap<String, Buffer>>>,
    hooks: Arc<RwLock<Hooks>>,
    current_buffer: Arc<RwLock<Option<String>>>,
    key_buffer: Arc<RwLock<KeyBuffer>>,
    /// This is for checking if a key is special,
    /// i.e. it performs editor state specific functionality like clearing the key buffer.
    ///
    /// This can only be a single key as only the current key will be checked.
    special_key_map: Arc<RwLock<KeyMap>>,
    main_key_map: Arc<RwLock<KeyMap>>,
    key_maps: Arc<RwLock<HashMap<Symbol, KeyMap>>>,
    broker_client: Arc<RwLock<BrokerClient>>,
    active_sessions: Arc<RwLock<Vec<usize>>>,
    command_bar: Arc<RwLock<CommandBar>>,
}

impl SessionState {
    pub fn new() -> Self {
        let mut hooks = Hooks::new();
        hooks.add_new_hook_kind(Symbol::intern("buffer-open"));

        let hooks = Arc::new(RwLock::new(hooks));
        let (sender, receiver) = tokio::sync::mpsc::channel(100);

        Self {
            buffers: Arc::new(RwLock::new(HashMap::new())),
            hooks,
            current_buffer: Arc::new(RwLock::new(None)),
            key_buffer: Arc::new(RwLock::new(KeyBuffer::new())),
            main_key_map: Arc::new(RwLock::new(KeyMap::new_sparse())),
            special_key_map: Arc::new(RwLock::new(KeyMap::new_sparse())),
            key_maps: Arc::new(RwLock::new(HashMap::new())),
            broker_client: Arc::new(RwLock::new(BrokerClient::new(0, sender, receiver))),
            active_sessions: Arc::new(RwLock::new(Vec::new())),
            command_bar: Arc::new(RwLock::new(CommandBar::new())),
        }
    }

    pub async fn add_session(session_id: usize) {
        let active_sessions = {
            let state = SessionState::get_state();
            state.read().await.active_sessions.clone()
        };
        active_sessions.write().await.push(session_id);
    }

    pub async fn remove_session(session_id: usize) {
        let active_sessions = {
            let state = SessionState::get_state();
            state.read().await.active_sessions.clone()
        };
        active_sessions.write().await.remove(session_id);
    }

    /// This function should only ever be called once.
    ///
    /// This function should always be called so that the backend has a way to communicate with the frontend.
    pub async fn set_broker_client(broker_client: BrokerClient) {
        let client = {
            let state = SessionState::get_state();
            state.read().await.broker_client.clone()
        };
        *client.write().await = broker_client;
    }

    pub async fn send_message(message: MessageKind) -> Result<(), Condition> {
        let clients = SessionState::get_state().read().await.active_sessions.clone();
        let client = SessionState::get_state().read().await.broker_client.clone();

        let mut broker_guard = client.write().await;

        for client in clients.read().await.iter() {
            broker_guard.send_async(message.clone(), *client)
                .await
                .map_err(|err| Condition::error(format!("{:?}", err)))?;
        }
        Ok(())
    }

    pub async fn get_buffers(&self) -> impl Deref<Target = HashMap<String, Buffer>> {
        self.buffers.read().await
    }
    pub async fn get_buffers_mut(&mut self) -> impl DerefMut<Target = HashMap<String, Buffer>> {
        self.buffers.write().await
    }

    pub async fn add_buffer(&mut self, name: &str, handle: BufferHandle) {
        self.buffers.write().await.insert(name.to_string(), Buffer::new(handle));
    }

    pub fn get_hooks(&self) -> &Arc<RwLock<Hooks>> {
        &self.hooks
    }

    pub async fn get_command_bar() -> Arc<RwLock<CommandBar>> {
        let command_bar = {
            let state = SessionState::get_state();
            state.read().await.command_bar.clone()
        };
        command_bar
    }

    pub async fn set_current_buffer(buffer_name: String) {
        let (buffers, current_buffer) = {
            let state = Self::get_state();
            let guard = state.read().await;
            (guard.buffers.clone(), guard.current_buffer.clone())
        };
        let mut different_buffer = true;
        if let Some(current_buffer) = current_buffer.read().await.as_ref() {
            if current_buffer.as_str() != buffer_name.as_str() {
                let buffer = {
                    buffers.read().await.get(&buffer_name).cloned()
                        .expect("current buffer somehow not not in the editor")
                };
                let major_mode_value = buffer.get_major_mode();
                let major_mode: Gc<MajorMode> = major_mode_value.clone()
                    .try_into_rust_type()
                    .expect("Somehow a non-major mode is in the place of a major mode");
                let lost_focus = major_mode.lose_focus();
                let result = lost_focus.call(&[major_mode_value]).await;
                match result {
                    Ok(_) => {}
                    Err(err) => {
                        error!("{}", err);
                    }
                }

                let minor_modes = buffer.get_minor_modes();
                for mode in minor_modes {
                    let minor_mode: Gc<MinorMode> = mode.try_into_rust_type()
                        .expect("Somehow a non-minor mode is in the place of a minor mode");
                    let lost_focus = minor_mode.lose_focus();
                    let result = lost_focus.call(&[mode]).await;
                    match result {
                        Ok(_) => {}
                        Err(err) => {
                            error!("{}", err);
                        }
                    }
                }
            } else {
                different_buffer = false;
            }
        }
        *current_buffer.write().await = Some(buffer_name.clone());
        if different_buffer {
            let buffer = {
                let result = buffers.read().await.get(&buffer_name).cloned();
                match result {
                    Some(buffer) => buffer,
                    None => {
                        error!("Buffer '{}' not found", buffer_name);
                        return;
                    }
                }
            };
            let major_mode_value = buffer.get_major_mode();
            match major_mode_value.clone().try_into_rust_type() {
                Ok(major_mode) => {
                    let major_mode: Gc<MajorMode> = major_mode;
                    let gain_focus = major_mode.gain_focus();
                    let result = gain_focus.call(&[major_mode_value]).await;
                    match result {
                        Ok(_) => {}
                        Err(err) => {
                            error!("{}", err);
                        }
                    }
                },
                Err(err) => {
                    error!("{}", err);
                }
            };

            let minor_modes = buffer.get_minor_modes();
            for mode in minor_modes {
                let minor_mode: Gc<MinorMode> = mode.try_into_rust_type()
                    .expect("Somehow a non-minor mode is in the place of a minor mode");
                let gain_focus = minor_mode.gain_focus();
                let result = gain_focus.call(&[mode]).await;
                match result {
                    Ok(_) => {}
                    Err(err) => {
                        error!("{}", err);
                    }
                }
            }
        }
    }

    pub async fn current_focused_buffer() -> Option<(String, Buffer)> {
        let (buffers, current_buffer) = {
            let state = Self::get_state();
            let guard = state.read().await;
            (guard.buffers.clone(), guard.current_buffer.clone())
        };
        if let Some(buffer) = current_buffer.read().await.as_ref() {
            let buffers = buffers.read().await;
            buffers.get(buffer.as_str()).cloned().map(|bfr| (buffer.clone(), bfr))
        } else {
            None
        }
    }

    pub async fn get_current_buffer() -> Option<Buffer> {
        let (buffers, current_buffer) = {
            let state = Self::get_state();
            let guard = state.read().await;
            (guard.buffers.clone(), guard.current_buffer.clone())
        };
        if let Some(buffer) = current_buffer.read().await.as_ref() {
            buffers.read().await.get(buffer).cloned()
        } else {
            None
        }
    }

    pub async fn add_keybinding(&self, keys: Vec<KeyPress>, command: Gc<Command>) {
        self.main_key_map.write().await.add_binding(keys, command);
    }

    pub async fn remove_keybinding(&self, keys: &[KeyPress]) {
        self.main_key_map.write().await.remove_binding(keys);
    }

    pub async fn add_special_keybinding(&self, keys: Vec<KeyPress>, command: Gc<Command>) {
        self.special_key_map.write().await.add_binding(keys, command);
    }

    pub async fn remove_special_keybinding(&self, keys: &[KeyPress]) {
        self.special_key_map.write().await.remove_binding(keys);
    }

    pub async fn add_keymap(&self, keymap_name: Symbol, keymap: Gc<SchemeKeyMap>) -> Result<(), String> {
        let keymap = keymap.make_keymap().await?;
        self.key_maps.write().await.insert(keymap_name, keymap);
        Ok(())
    }

    pub async fn remove_keymap(&self, keymap_name: Symbol) {
        let mut guard = self.key_maps.write().await;
        guard.remove(&keymap_name);
    }

    /// Returns: `true` if a mapping has been found, `false` if a mapping was not found
    async fn try_process_keypress(keys: &[KeyPress], map: &KeyMap) -> KeyPressCommandResult {
        let mut flush = false;
        let mut found = false;
        if let Some(command) = map.lookup(&keys).cloned() {
            let proc = command.command().clone();
            let args = keys.iter().map(|press| {
                Value::from(Record::from_rust_type((*press).clone()))
            }).collect::<Vec<Value>>();

            let list = lists::slice_to_list(&args);

            match proc.call(&[list]).await {
                Ok(value) => {
                    if value.is_empty() {
                        flush = true;
                        found = true;
                    } else if !value.is_empty() {
                        let unpack = value[0].clone().unpack();
                        match unpack {
                            UnpackedValue::Boolean(b) => {
                                if b {
                                    found = true;
                                    flush = true;
                                } else {
                                    found = false;
                                    flush = false;
                                }
                            }
                            _ => {
                                flush = true;
                                found = true;
                            }
                        }
                    }
                },
                Err(e) => {
                    let key_string = keys.iter()
                        .map(|press| press.to_string())
                        .collect::<Vec<String>>();
                    let key_string = key_string.join(" ");
                    error!("{key_string}:\n{e}");
                }
            }
        } else {
            found = false;
            flush = false;
        }
        KeyPressCommandResult::new(flush, found)
    }

    pub async fn process_keypress(keypress: KeyPress) {
        let keypress = keypress.canonicalize(SessionState::get_keyboard_region());
        let (key_buffer, main_map, maps, special) = {
            let state = Self::get_state();
            let guard = state.read().await;
            (guard.key_buffer.clone(), guard.main_key_map.read().await.clone(), guard.key_maps.read().await.clone(), guard.special_key_map.read().await.clone())
        };
        let result = Self::try_process_keypress(&vec![keypress.clone()], &special).await;
        if result.found {
            return;
        }

        Self::add_to_key_buffer(keypress).await;
        let mut clear_key_buffer = false;
        let keys = key_buffer.read().await.get().to_vec();
        let result = Self::try_process_keypress(&keys, &main_map).await;
        if result.found && result.flush {
            clear_key_buffer = true;
        }
        if !result.found {
            for (_, keymap) in maps.iter() {
                let result = Self::try_process_keypress(&keys, keymap).await;
                if result.found {
                    if result.flush {
                        clear_key_buffer = true;
                    }
                    break;
                }
            }
        }

        if clear_key_buffer {
            Self::flush_key_buffer().await;
        }
    }

    async fn get_key_buffer() -> Arc<RwLock<KeyBuffer>> {
        let key_buffer = {
            let state = Self::get_state();
            let guard = state.read().await;
            guard.key_buffer.clone()
        };
        key_buffer
    }
    
    pub async fn flush_key_buffer() {
        let key_buffer = Self::get_key_buffer().await;
        key_buffer.write().await.clear();
    }

    pub async fn add_to_key_buffer(key_press: KeyPress) {
        let key_buffer = Self::get_key_buffer().await;
        key_buffer.write().await.push(key_press);
    }

    pub async fn emit_hook(hook_name: Symbol, args: &[Value]) -> Result<(), Condition> {
        let args = args.to_vec();
        tokio::task::spawn(async move {
            let state = SessionState::get_state();

            let hooks = state.read().await.hooks.clone();
            let hooks = hooks.read().await;
            match hooks.execute_hook(&hook_name, &args).await {
                Err(err) => {
                    error!("{err}");
                }
                _ => {}
            }
        });
        
        Ok(())
    }

    pub async fn emit_hook_self(&mut self, hook_name: Symbol, args: &[Value]) -> Result<(), Condition> {
        let hooks = self.hooks.read().await;
        hooks.execute_hook(&hook_name, args).await?;
        Ok(())
    }

    pub fn get_state() -> Arc<RwLock<SessionState>> {
        STATE.clone()
    }

    pub fn set_keyboard_region(new_region: KeyboardRegion) {
        unsafe {
            KEYBOARD_REGION = new_region;
        }
    }

    pub fn get_keyboard_region() -> KeyboardRegion {
        unsafe {
            KEYBOARD_REGION
        }
    }
}

static STATE: LazyLock<Arc<RwLock<SessionState>>> = LazyLock::new(|| Arc::new(RwLock::new(SessionState::new())));

static mut KEYBOARD_REGION: KeyboardRegion = KeyboardRegion::EnglishUS;

#[bridge(name = "create-hook", lib = "(koru-session)")]
pub async fn create_hook(args: &[Value]) -> Result<Vec<Value>, Condition> {
    let Some((hook_name, _)) = args.split_first() else {
        return Err(Condition::wrong_num_of_args(1, args.len()))
    };
    let hook_name: Symbol = hook_name.clone().try_into()?;

    let state = SessionState::get_state();

    let hooks = state.read().await.hooks.clone();
    hooks.write().await.add_new_hook_kind(hook_name);

    Ok(Vec::new())
}

#[bridge(name = "destroy-hook", lib = "(koru-session)")]
pub async fn destroy_hook(args: &[Value]) -> Result<Vec<Value>, Condition> {
    let Some((hook_name, _)) = args.split_first() else {
        return Err(Condition::wrong_num_of_args(1, args.len()))
    };
    let hook_name: Symbol = hook_name.clone().try_into()?;

    let state = SessionState::get_state();

    let hooks = state.read().await.hooks.clone();
    hooks.write().await.remove_hook_kind(&hook_name);

    Ok(Vec::new())
}

#[bridge(name = "add-hook", lib = "(koru-session)")]
pub async fn add_hook(args: &[Value]) -> Result<Vec<Value>, Condition> {
    let Some((hook_name_kind, rest)) = args.split_first() else {
        return Err(Condition::wrong_num_of_args(3, args.len()))
    };
    let Some((hook_name, rest)) = rest.split_first() else {
        return Err(Condition::wrong_num_of_args(3, args.len()))
    };
    let Some((procedure, _)) = rest.split_first() else {
        return Err(Condition::wrong_num_of_args(3, args.len()))
    };
    let hook_name_kind: Symbol = hook_name_kind.clone().try_into()?;
    let hook_name: Symbol = hook_name.clone().try_into()?;
    let hook: Procedure = procedure.clone().try_into()?;

    let state = SessionState::get_state();

    let hooks = state.read().await.hooks.clone();
    hooks.write().await.add_new_hook(hook_name_kind, hook_name, hook);
    Ok(Vec::new())
}


#[bridge(name = "remove-hook", lib = "(koru-session)")]
pub async fn remove_hook(args: &[Value]) -> Result<Vec<Value>, Condition> {
    let Some((hook_name_kind, rest)) = args.split_first() else {
        return Err(Condition::wrong_num_of_args(2, args.len()))
    };
    let Some((hook_name, _)) = rest.split_first() else {
        return Err(Condition::wrong_num_of_args(2, args.len()))
    };
    let hook_name_kind: Symbol = hook_name_kind.clone().try_into()?;
    let hook_name: Symbol = hook_name.clone().try_into()?;

    let state = SessionState::get_state();

    let hooks = state.read().await.hooks.clone();
    hooks.write().await.remove_hook(&hook_name_kind, &hook_name);
    Ok(Vec::new())
}

#[bridge(name = "emit-hook", lib = "(koru-session)")]
pub async fn emit_hook(args: &[Value]) -> Result<Vec<Value>, Condition> {
    let Some((hook_name_kind, rest)) = args.split_first() else {
        return Err(Condition::wrong_num_of_args(1, args.len()))
    };
    let hook_name: Symbol = hook_name_kind.clone().try_into()?;

    SessionState::emit_hook(hook_name, rest).await?;
    Ok(Vec::new())
}

#[bridge(name = "major-mode-set!", lib = "(koru-buffer)")]
pub async fn set_major_mode(args: &[Value]) -> Result<Vec<Value>, Condition> {
    let Some((buffer_name, rest)) = args.split_first() else {
        return Err(Condition::wrong_num_of_args(2, args.len()))
    };
    let Some((major_mode, _)) = rest.split_first() else {
        return Err(Condition::wrong_num_of_args(2, args.len()))
    };
    let buffer_name: String = buffer_name.clone().try_into()?;
    let _: Gc<MajorMode> = major_mode.try_into_rust_type()?;

    let state = SessionState::get_state();
    let guard = state.read().await;
    let mut buffer_guard = guard.buffers.write().await;
    let Some(buffer) = buffer_guard.get_mut(&buffer_name) else {
        return Err(Condition::error(String::from("Buffer not found")));
    };
    
    buffer.set_major_mode(major_mode.clone()).await?;
    
    Ok(Vec::new())
}

#[bridge(name = "minor-mode-add", lib = "(koru-buffer)")]
pub async fn add_minor_mode(args: &[Value]) -> Result<Vec<Value>, Condition> {
    let Some((buffer_name, rest)) = args.split_first() else {
        return Err(Condition::wrong_num_of_args(2, args.len()))
    };
    let Some((minor_mode, _)) = rest.split_first() else {
        return Err(Condition::wrong_num_of_args(2, args.len()))
    };
    let buffer_name: String = buffer_name.clone().try_into()?;
    let _: Gc<MinorMode> = minor_mode.try_into_rust_type()?;

    let state = SessionState::get_state();
    let guard = state.read().await;
    let mut buffer_guard = guard.buffers.write().await;
    let Some(buffer) = buffer_guard.get_mut(&buffer_name) else {
        return Err(Condition::error(String::from("Buffer not found")));
    };

    buffer.add_minor_mode(minor_mode.clone()).await?;

    Ok(Vec::new())
}

#[bridge(name = "minor-mode-get", lib = "(koru-buffer)")]
pub async fn get_minor_mode(minor_mode_name: &Value) -> Result<Vec<Value>, Condition> {
    let minor_mode_name: Symbol = minor_mode_name.clone().try_into()?;
    let current_buffer = SessionState::get_current_buffer().await
        .ok_or(Condition::error(String::from("no buffer currently focused")))?;

    let minor_mode = current_buffer.get_minor_mode(minor_mode_name).await
        .ok_or(Condition::error(String::from("minor mode not found")))?;

    Ok(vec![minor_mode])
}

#[bridge(name = "current-major-mode", lib = "(koru-buffer)")]
pub async fn get_current_major_mode() -> Result<Vec<Value>, Condition> {
    let Some(buffer) = SessionState::get_current_buffer().await else {
        return Err(Condition::error(String::from("Buffer not found")));
    };

    Ok(vec![buffer.get_major_mode()])
}

#[bridge(name = "current-buffer-name", lib = "(koru-buffer)")]
pub async fn get_current_buffer_name() -> Result<Vec<Value>, Condition> {
    let Some((name, _)) = SessionState::current_focused_buffer().await else {
        return Ok(vec![Value::null()])
    };

    Ok(vec![Value::from(name)])
}

#[bridge(name = "add-key-binding", lib = "(koru-session)")]
pub async fn add_keymaping(args: &[Value]) -> Result<Vec<Value>, Condition> {
    let Some((key_string, rest)) = args.split_first() else {
        return Err(Condition::wrong_num_of_args(2, args.len()))
    };
    let Some((command, _)) = rest.split_first() else {
        return Err(Condition::wrong_num_of_args(2, args.len()))
    };

    let key_string: String = key_string.clone().try_into()?;
    let command: Gc<Command> = command.try_into_rust_type()?;

    let key_seq = key_string.split_whitespace()
        .map(|s| {
            KeyPress::from_string(s)
        })
        .collect::<Option<Vec<KeyPress>>>();

    let key_seq = match key_seq {
        Some(key_seq) => key_seq,
        None => {
            return Err(Condition::error(String::from("Invalid key in sequence")))
        }
    };

    let state = SessionState::get_state();
    let guard = state.read().await;
    guard.add_keybinding(key_seq, command).await;

    Ok(Vec::new())
}

#[bridge(name = "remove-key-binding", lib = "(koru-session)")]
pub async fn remove_keymaping(args: &[Value]) -> Result<Vec<Value>, Condition> {
    let Some((key_string, _)) = args.split_first() else {
        return Err(Condition::wrong_num_of_args(1, args.len()))
    };

    let key_string: String = key_string.clone().try_into()?;

    let key_seq = key_string.split_whitespace()
        .map(|s| {
            KeyPress::from_string(s)
        })
        .collect::<Option<Vec<KeyPress>>>();

    let key_seq = match key_seq {
        Some(key_seq) => key_seq,
        None => {
            return Err(Condition::error(String::from("Invalid key in sequence")))
        }
    };

    let state = SessionState::get_state();
    let guard = state.read().await;
    guard.remove_keybinding(&key_seq).await;

    Ok(Vec::new())
}

#[bridge(name = "add-special-key-binding", lib = "(koru-session)")]
pub async fn add_special_keymaping(args: &[Value]) -> Result<Vec<Value>, Condition> {
    let Some((key_string, rest)) = args.split_first() else {
        return Err(Condition::wrong_num_of_args(2, args.len()))
    };
    let Some((command, _)) = rest.split_first() else {
        return Err(Condition::wrong_num_of_args(2, args.len()))
    };

    let key_string: String = key_string.clone().try_into()?;
    let command: Gc<Command> = command.try_into_rust_type()?;

    let key_seq = key_string.split_whitespace()
        .map(|s| {
            KeyPress::from_string(s)
        })
        .collect::<Option<Vec<KeyPress>>>();

    let key_seq = match key_seq {
        Some(key_seq) => key_seq,
        None => {
            return Err(Condition::error(String::from("Invalid key in sequence")))
        }
    };

    let state = SessionState::get_state();
    let guard = state.read().await;
    guard.add_special_keybinding(key_seq, command).await;

    Ok(Vec::new())
}

#[bridge(name = "remove-special-key-binding", lib = "(koru-session)")]
pub async fn remove_special_keymaping(args: &[Value]) -> Result<Vec<Value>, Condition> {
    let Some((key_string, _)) = args.split_first() else {
        return Err(Condition::wrong_num_of_args(1, args.len()))
    };

    let key_string: String = key_string.clone().try_into()?;

    let key_seq = key_string.split_whitespace()
        .map(|s| {
            KeyPress::from_string(s)
        })
        .collect::<Option<Vec<KeyPress>>>();

    let key_seq = match key_seq {
        Some(key_seq) => key_seq,
        None => {
            return Err(Condition::error(String::from("Invalid key in sequence")))
        }
    };

    let state = SessionState::get_state();
    let guard = state.read().await;
    guard.remove_special_keybinding(&key_seq).await;

    Ok(Vec::new())
}

#[bridge(name = "add-key-map", lib = "(koru-session)")]
pub async fn add_keymap(args: &[Value]) -> Result<Vec<Value>, Condition> {
    let Some((keymap_name, rest)) = args.split_first() else {
        return Err(Condition::wrong_num_of_args(2, args.len()))
    };
    let Some((keymap, _)) = rest.split_first() else {
        return Err(Condition::wrong_num_of_args(2, args.len()))
    };
    let keymap_name: Symbol = keymap_name.clone().try_into()?;
    let keymap: Gc<SchemeKeyMap> = keymap.try_into_rust_type()?;
    let state = SessionState::get_state();
    let guard = state.read().await;
    match guard.add_keymap(keymap_name, keymap).await {
        Err(msg) => Err(Condition::error(msg)),
        Ok(()) => Ok(Vec::new())
    }
}

#[bridge(name = "remove-key-map", lib = "(koru-session)")]
pub async fn remove_keymap(keymap_name: &Value) -> Result<Vec<Value>, Condition> {
    let keymap_name: Symbol = keymap_name.clone().try_into()?;
    let state = SessionState::get_state();
    let guard = state.read().await;
    guard.remove_keymap(keymap_name).await;
    Ok(Vec::new())
}

#[bridge(name = "flush-key-buffer", lib = "(koru-session)")]
pub async fn flush_keybuffer() -> Result<Vec<Value>, Condition> {
    let keybuffer = {
        let state = SessionState::get_state();
        let guard = state.read().await;
        guard.key_buffer.clone()
    };

    keybuffer.write().await.clear();
    Ok(Vec::new())
}

#[bridge(name = "buffer-change-focus", lib = "(koru-buffer)")]
pub async fn change_current_buffer(name: &Value) -> Result<Vec<Value>, Condition> {
    let name: String = name.clone().try_into()?;
    SessionState::set_current_buffer(name).await;
    Ok(Vec::new())
}

#[bridge(name = "command-bar-left", lib = "(koru-session)")]
pub async fn command_bar_left() -> Result<Vec<Value>, Condition> {
    let command_buffer = SessionState::get_command_bar().await;
    command_buffer.write().await.cursor_left();
    Ok(Vec::new())
}

#[bridge(name = "command-bar-right", lib = "(koru-session)")]
pub async fn command_bar_right() -> Result<Vec<Value>, Condition> {
    let command_buffer = SessionState::get_command_bar().await;
    command_buffer.write().await.cursor_right();
    Ok(Vec::new())
}

#[bridge(name = "command-bar-delete-back", lib = "(koru-session)")]
pub async fn command_bar_delete_backward() -> Result<Vec<Value>, Condition> {
    let command_buffer = SessionState::get_command_bar().await;
    command_buffer.write().await.delete_backward();
    Ok(Vec::new())
}

#[bridge(name = "command-bar-delete-forward", lib = "(koru-session)")]
pub async fn command_bar_delete_forward() -> Result<Vec<Value>, Condition> {
    let command_buffer = SessionState::get_command_bar().await;
    command_buffer.write().await.delete_forward();
    Ok(Vec::new())
}

#[bridge(name = "command-bar-take", lib = "(koru-session)")]
pub async fn command_bar_take() -> Result<Vec<Value>, Condition> {
    let command_buffer = SessionState::get_command_bar().await;
    let string = command_buffer.write().await.take();
    Ok(vec![Value::from(string)])
}

#[bridge(name = "command-bar-get", lib = "(koru-session)")]
pub async fn command_bar_get() -> Result<Vec<Value>, Condition> {
    let command_buffer = SessionState::get_command_bar().await;
    let string = command_buffer.read().await.get();
    Ok(vec![Value::from(string)])
}

#[bridge(name = "command-bar-insert", lib = "(koru-session)")]
pub async fn command_bar_insert(string: &Value) -> Result<Vec<Value>, Condition> {
    let string: String = string.clone().try_into()?;
    let command_buffer = SessionState::get_command_bar().await;
    command_buffer.write().await.insert(&string);
    Ok(Vec::new())
}

#[bridge(name = "command-bar-insert-key", lib = "(koru-session)")]
pub async fn command_bar_insert_key(key_seq: &Value) -> Result<Vec<Value>, Condition> {
    let key_press = {
        let key_sequence = key_seq.clone().unpack();
        match key_sequence {
            UnpackedValue::Pair(pair) => {
                let cdr = pair.cdr().clone();
                if !cdr.is_null() {
                    // Skip if the key sequence is 2 or greater
                    return Ok(vec![Value::from(false)]);
                }
                let key = pair.car().clone();
                let key: Gc<KeyPress> = key.try_into_rust_type()?;
                (*key).clone()
            }
            _ => {
                return Err(Condition::type_error("List", key_sequence.type_name()))
            }
        }
    };

    if !key_press.modifiers.is_empty() {
        return Ok(vec![Value::from(false)]);
    }

    let command_buffer = SessionState::get_command_bar().await;
    match key_press.key {
        KeyValue::CharacterKey(str) => {
            command_buffer.write().await.insert(&str);
        }
        _ => {}
    }

    Ok(vec![Value::from(true)])
}

#[bridge(name = "command-bar-show", lib = "(koru-session)")]
pub async fn command_bar_show() -> Result<Vec<Value>, Condition> {
    SessionState::send_message(MessageKind::BackEnd(BackendMessage::ShowCommandBar)).await?;
    Ok(Vec::new())
}

#[bridge(name = "command-bar-hide", lib = "(koru-session)")]
pub async fn command_bar_hide() -> Result<Vec<Value>, Condition> {
    SessionState::send_message(MessageKind::BackEnd(BackendMessage::HideCommandBar)).await?;
    Ok(Vec::new())
}

#[bridge(name = "command-bar-update", lib = "(koru-session)")]
pub async fn command_bar_update(args: &[Value]) -> Result<Vec<Value>, Condition> {
    let Some((prefix, rest)) = args.split_first() else {
        let command_buffer = SessionState::get_command_bar().await;
        let string = command_buffer.read().await.get();
        SessionState::send_message(MessageKind::BackEnd(BackendMessage::UpdateCommandBar {
            body: string,
            prefix: String::new(),
            suffix: String::new(),
        })).await?;
        return Ok(Vec::new());
    };
    let prefix: String = prefix.clone().try_into()?;
    let Some((suffix, rest)) = rest.split_first() else {
        let command_buffer = SessionState::get_command_bar().await;
        let string = command_buffer.read().await.get();
        SessionState::send_message(MessageKind::BackEnd(BackendMessage::UpdateCommandBar {
            prefix,
            body: string,
            suffix: String::new(),
            
        })).await?;
        return Ok(Vec::new());
    };
    let suffix: String = suffix.clone().try_into()?;

    let command_buffer = SessionState::get_command_bar().await;
    let string = command_buffer.read().await.get();
    SessionState::send_message(MessageKind::BackEnd(BackendMessage::UpdateCommandBar {
        prefix,
        body: string,
        suffix,
    })).await?;
    
    Ok(Vec::new())
}

#[bridge(name = "crash", lib = "(koru-session)")]
pub fn crash() -> Result<Vec<Value>, Condition> {
    error!("Crash");
    panic!("crash");
}