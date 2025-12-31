mod buffer;
mod keymap;

use std::ops::DerefMut;
use std::ops::Deref;
pub use buffer::*;

use std::collections::HashMap;
use std::sync::{Arc, LazyLock};
use scheme_rs::exceptions::Condition;
use scheme_rs::gc::Gc;
use scheme_rs::lists;
use scheme_rs::proc::Procedure;
use scheme_rs::records::Record;
use scheme_rs::registry::bridge;
use scheme_rs::value::{UnpackedValue, Value};
use tokio::sync::{Mutex, RwLock};
use keypress_localize::KeyboardRegion;
use crate::kernel::buffer::{BufferHandle};
use crate::kernel::input::{KeyBuffer, KeyPress};
use crate::kernel::scheme_api::command::Command;
use crate::kernel::scheme_api::major_mode::MajorMode;
use crate::kernel::scheme_api::session::keymap::SchemeKeyMap;
use crate::keymap::KeyMap;

pub struct Hooks {
    hooks: HashMap<String, HashMap<String, Procedure>>,
}

impl Hooks {
    fn new() -> Self {
        Hooks {
            hooks: HashMap::new(),
        }
    }

    pub fn add_new_hook_kind(&mut self, name: String) {
        self.hooks.insert(name, HashMap::new());
    }

    pub fn remove_hook_kind(&mut self, name: &str) {
        self.hooks.remove(name);
    }

    pub fn add_new_hook(&mut self, hook_name: &str, name: String, procedure: Procedure) {
        let Some(hooks) = self.hooks.get_mut(hook_name) else {
            panic!("Unknown hook {}", hook_name);
        };
        hooks.insert(name, procedure);
    }

    pub fn remove_hook(&mut self, hook_name: &str, name: &str) {
        if let Some(hooks) = self.hooks.get_mut(hook_name) {
            hooks.remove(name);
        }
    }

    pub async fn execute_hook(&self, hook_name: &str, args: &[Value]) -> Result<(), Condition> {
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

pub struct SessionState {
    buffers: RwLock<HashMap<String, Buffer>>,
    hooks: Arc<RwLock<Hooks>>,
    current_buffer: Option<String>,
    key_buffer: Arc<RwLock<KeyBuffer>>,
    /// This is for checking if a key is special,
    /// i.e. it performs editor state specific functionality like clearing the key buffer.
    ///
    /// This can only be a single key as only the current key will be checked.
    special_key_map: Arc<RwLock<KeyMap>>,
    main_key_map: Arc<RwLock<KeyMap>>,
    key_maps: Arc<RwLock<HashMap<String, KeyMap>>>,
}

impl SessionState {
    pub fn new() -> Self {
        let mut hooks = Hooks::new();
        hooks.add_new_hook_kind(String::from("file-open"));

        let hooks = Arc::new(RwLock::new(hooks));

        Self {
            buffers: RwLock::new(HashMap::new()),
            hooks,
            current_buffer: None,
            key_buffer: Arc::new(RwLock::new(KeyBuffer::new())),
            main_key_map: Arc::new(RwLock::new(KeyMap::new_sparse())),
            special_key_map: Arc::new(RwLock::new(KeyMap::new_sparse())),
            key_maps: Arc::new(RwLock::new(HashMap::new())),
        }
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

    pub fn set_current_buffer(&mut self, buffer_name: String) {
        self.current_buffer = Some(buffer_name);
    }

    pub fn current_focused_buffer(&self) -> Option<&String> {
        self.current_buffer.as_ref()
    }

    pub async fn get_current_buffer(&self) -> Option<Buffer> {
        if let Some(buffer) = self.current_buffer.as_ref() {
            self.buffers.read().await.get(buffer).cloned()
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

    pub async fn add_keymap(&self, keymap_name: String, keymap: Gc<SchemeKeyMap>) -> Result<(), String> {
        let keymap = keymap.make_keymap().await?;
        self.key_maps.write().await.insert(keymap_name, keymap);
        Ok(())
    }

    pub async fn remove_keymap(&self, keymap_name: &str) {
        self.key_maps.write().await.remove(keymap_name);
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
                    println!("{}", e);
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
            (guard.key_buffer.clone(), guard.main_key_map.clone(), guard.key_maps.clone(), guard.special_key_map.clone())
        };
        let result = Self::try_process_keypress(&vec![keypress.clone()], &*special.read().await).await;
        if result.found {
            return;
        }

        Self::add_to_key_buffer(keypress).await;
        let mut clear_key_buffer = false;
        let keys = key_buffer.read().await.get().to_vec();
        let result = Self::try_process_keypress(&keys, &*main_map.read().await).await;
        if result.found && result.flush {
            clear_key_buffer = true;
        }
        if !result.found {
            for (name, keymap) in maps.read().await.iter() {
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
    let hook_name: String = hook_name.clone().try_into()?;

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
    let hook_name: String = hook_name.clone().try_into()?;

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
    let hook_name_kind: String = hook_name_kind.clone().try_into()?;
    let hook_name: String = hook_name.clone().try_into()?;
    let hook: Procedure = procedure.clone().try_into()?;

    let state = SessionState::get_state();

    let hooks = state.read().await.hooks.clone();
    hooks.write().await.add_new_hook(&hook_name_kind, hook_name, hook);
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
    let hook_name_kind: String = hook_name_kind.clone().try_into()?;
    let hook_name: String = hook_name.clone().try_into()?;

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
    let hook_name: String = hook_name_kind.clone().try_into()?;

    let state = SessionState::get_state();

    let hooks = state.read().await.hooks.clone();
    let hooks = hooks.read().await;
    hooks.execute_hook(&hook_name, rest).await?;
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
    
    buffer.set_major_mode(major_mode.clone());
    
    Ok(Vec::new())
}

#[bridge(name = "current-major-mode", lib = "(koru-buffer)")]
pub async fn get_current_major_mode() -> Result<Vec<Value>, Condition> {
    let state = SessionState::get_state();
    let guard = state.read().await;
    let Some(buffer) = guard.get_current_buffer().await else {
        return Err(Condition::error(String::from("Buffer not found")));
    };

    Ok(vec![buffer.get_major_mode()])
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

#[bridge(name = "add-key-map", lib = "(koru-session)")]
pub async fn add_keymap(args: &[Value]) -> Result<Vec<Value>, Condition> {
    let Some((keymap_name, rest)) = args.split_first() else {
        return Err(Condition::wrong_num_of_args(2, args.len()))
    };
    let Some((keymap, _)) = rest.split_first() else {
        return Err(Condition::wrong_num_of_args(2, args.len()))
    };
    let keymap_name: String = keymap_name.clone().try_into()?;
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
    let keymap_name: String = keymap_name.clone().try_into()?;
    let state = SessionState::get_state();
    let guard = state.read().await;
    guard.remove_keymap(&keymap_name).await;
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
    let state = SessionState::get_state();
    let mut guard = state.write().await;
    guard.current_buffer = Some(name);
    Ok(Vec::new())
}