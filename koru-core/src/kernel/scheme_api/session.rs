mod buffer;

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
use scheme_rs::value::Value;
use tokio::sync::{Mutex, RwLock};
use crate::kernel::buffer::{BufferHandle};
use crate::kernel::input::{KeyBuffer, KeyPress};
use crate::kernel::scheme_api::command::Command;
use crate::kernel::scheme_api::major_mode::MajorMode;
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


pub struct SessionState {
    buffers: RwLock<HashMap<String, Buffer>>,
    hooks: Arc<RwLock<Hooks>>,
    current_buffer: Option<String>,
    key_buffer: RwLock<KeyBuffer>,
    key_map: KeyMap,
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
            key_buffer: RwLock::new(KeyBuffer::new()),
            key_map: KeyMap::new_sparse(),
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

    pub fn add_keybinding(&mut self, keys: Vec<KeyPress>, command: Gc<Command>) {
        self.key_map.add_binding(keys, command);
    }

    pub fn remove_keybinding(&mut self, keys: &[KeyPress]) {
        self.key_map.remove_binding(keys);
    }

    pub async fn process_keypress(&self, keypress: KeyPress) {
        self.key_buffer.write().await.push(keypress);
        let mut clear_key_buffer = false;
        if let Some(command) = self.key_map.lookup(self.key_buffer.read().await.get()).cloned() {
            let proc = command.command().clone();
            let args = self.key_buffer.read().await.get().iter().map(|press| {
                Value::from(Record::from_rust_type(*press))
            }).collect::<Vec<Value>>();

            let list = lists::slice_to_list(&args);

            match proc.call(&[list]).await {
                Ok(_) => {},
                Err(e) => {
                    println!("{}", e);
                }
            }
            clear_key_buffer = true;
        }
        if clear_key_buffer {
            self.key_buffer.write().await.clear();
        }
    }
    
    pub async fn flush_key_buffer(&self) {
        self.key_buffer.write().await.clear();
    }

    pub fn get_state() -> Arc<RwLock<SessionState>> {
        STATE.clone()
    }
}

static STATE: LazyLock<Arc<RwLock<SessionState>>> = LazyLock::new(|| Arc::new(RwLock::new(SessionState::new())));


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

#[bridge(name = "major-mode-set!", lib = "(koru-session)")]
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

#[bridge(name = "current-major-mode", lib = "(koru-session)")]
pub async fn get_current_major_mode() -> Result<Vec<Value>, Condition> {
    let state = SessionState::get_state();
    let guard = state.read().await;
    let Some(buffer) = guard.get_current_buffer().await else {
        return Err(Condition::error(String::from("Buffer not found")));
    };

    Ok(vec![buffer.get_major_mode()])
}

#[bridge(name = "add-key-mapping", lib = "(koru-session)")]
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
    let mut guard = state.write().await;
    guard.key_map.add_binding(key_seq, command);

    Ok(Vec::new())
}