use std::collections::HashMap;
use std::sync::{Arc, LazyLock};
use scheme_rs::exceptions::Condition;
use scheme_rs::gc::Gc;
use scheme_rs::proc::Procedure;
use scheme_rs::registry::bridge;
use scheme_rs::value::Value;
use tokio::sync::Mutex;
use crate::kernel::buffer::{BufferHandle, Cursor, CursorDirection, GridCursor, LeadingEdge};
use crate::kernel::scheme_api::major_mode::MajorMode;
use crate::styled_text::StyledFile;

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


#[derive(Clone)]
pub struct Buffer {
    major_mode: Value,
    handle: BufferHandle,
    cursors: Vec<Cursor>,
}

impl Buffer {
    fn new(handle: BufferHandle) -> Self {
        Buffer {
            major_mode: Value::undefined(),
            handle,
            cursors: vec![Cursor::new_main(GridCursor::default(), LeadingEdge::Start)]
        }
    }

    pub fn set_major_mode(&mut self, major_mode: Value) {
        self.major_mode = major_mode;
    }

    pub fn get_handle(&self) -> BufferHandle {
        self.handle.clone()
    }
    
    pub async fn get_styled_text(&self) -> StyledFile {
        let text = self.handle.get_text().await;
        let file = StyledFile::from(text);
        file.place_cursors(&self.cursors)
    }
    pub fn get_major_mode(&self) -> Value {
        self.major_mode.clone()
    }
    
    pub async fn move_cursors(&mut self, direction: CursorDirection) {
        let handle = self.handle.clone();
        self.cursors = handle.move_cursors(self.cursors.clone(), direction).await
    }
}

pub struct SessionState {
    buffers: HashMap<String, Buffer>,
    hooks: Arc<Mutex<Hooks>>,
    current_buffer: Option<String>,
}

impl SessionState {
    pub fn new() -> Self {
        let mut hooks = Hooks::new();
        hooks.add_new_hook_kind(String::from("file-open"));

        let hooks = Arc::new(Mutex::new(hooks));

        Self {
            buffers: HashMap::new(),
            hooks,
            current_buffer: None,
        }
    }

    pub fn get_buffers(&mut self) -> &mut HashMap<String, Buffer> {
        &mut self.buffers
    }
    
    pub fn add_buffer(&mut self, name: &str, handle: BufferHandle) {
        self.buffers.insert(name.to_string(), Buffer::new(handle));
    }

    pub fn get_hooks(&self) -> &Arc<Mutex<Hooks>> {
        &self.hooks
    }
    
    pub fn set_current_buffer(&mut self, buffer_name: String) {
        self.current_buffer = Some(buffer_name);
    }
    
    pub fn current_focused_buffer(&self) -> Option<&String> {
        self.current_buffer.as_ref()
    }
    
    pub fn get_current_buffer(&self) -> Option<&Buffer> {
        if let Some(buffer) = self.current_buffer.as_ref() {
            self.buffers.get(buffer)
        } else {
            None
        }
    }
    
    pub fn get_current_buffer_mut(&mut self) -> Option<&mut Buffer> {
        if let Some(buffer) = self.current_buffer.as_mut() {
            self.buffers.get_mut(buffer)
        } else {
            None
        }
    }
    
    
    pub fn get_state() -> Arc<Mutex<SessionState>> {
        STATE.clone()
    }
}




static STATE: LazyLock<Arc<Mutex<SessionState>>> = LazyLock::new(|| Arc::new(Mutex::new(SessionState::new())));

#[bridge(name = "create-hook", lib = "(koru-session)")]
pub async fn create_hook(args: &[Value]) -> Result<Vec<Value>, Condition> {
    let Some((hook_name, _)) = args.split_first() else {
        return Err(Condition::wrong_num_of_args(1, args.len()))
    };
    let hook_name: String = hook_name.clone().try_into()?;

    let state = SessionState::get_state();

    let hooks = state.lock().await.hooks.clone();
    hooks.lock().await.add_new_hook_kind(hook_name);

    Ok(Vec::new())
}

#[bridge(name = "destroy-hook", lib = "(koru-session)")]
pub async fn destroy_hook(args: &[Value]) -> Result<Vec<Value>, Condition> {
    let Some((hook_name, _)) = args.split_first() else {
        return Err(Condition::wrong_num_of_args(1, args.len()))
    };
    let hook_name: String = hook_name.clone().try_into()?;

    let state = SessionState::get_state();

    let hooks = state.lock().await.hooks.clone();
    hooks.lock().await.remove_hook_kind(&hook_name);

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

    let hooks = state.lock().await.hooks.clone();
    hooks.lock().await.add_new_hook(&hook_name_kind, hook_name, hook);
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

    let hooks = state.lock().await.hooks.clone();
    hooks.lock().await.remove_hook(&hook_name_kind, &hook_name);
    Ok(Vec::new())
}

#[bridge(name = "emit-hook", lib = "(koru-session)")]
pub async fn emit_hook(args: &[Value]) -> Result<Vec<Value>, Condition> {
    let Some((hook_name_kind, rest)) = args.split_first() else {
        return Err(Condition::wrong_num_of_args(1, args.len()))
    };
    let hook_name: String = hook_name_kind.clone().try_into()?;

    let state = SessionState::get_state();

    let hooks = state.lock().await.hooks.clone();
    let hooks = hooks.lock().await;
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
    let mut guard = state.lock().await;
    let Some(buffer) = guard.buffers.get_mut(&buffer_name) else {
        return Err(Condition::error(String::from("Buffer not found")));
    };
    
    buffer.set_major_mode(major_mode.clone());
    
    Ok(Vec::new())
}

#[bridge(name = "move-cursors-up", lib = "(koru-session)")]
pub async fn move_cursors_up() -> Result<Vec<Value>, Condition> {
    let state = SessionState::get_state();
    let mut guard = state.lock().await;
    let Some(buffer) = guard.get_current_buffer_mut() else {
        return Err(Condition::error(String::from("Buffer not found")));
    };
    buffer.move_cursors(CursorDirection::Up).await;
    Ok(Vec::new())
}

#[bridge(name = "move-cursors-down", lib = "(koru-session)")]
pub async fn move_cursors_down() -> Result<Vec<Value>, Condition> {
    let state = SessionState::get_state();
    let mut guard = state.lock().await;
    let Some(buffer) = guard.get_current_buffer_mut() else {
        return Err(Condition::error(String::from("Buffer not found")));
    };
    buffer.move_cursors(CursorDirection::Down).await;
    Ok(Vec::new())
}

#[bridge(name = "move-cursors-left", lib = "(koru-session)")]
pub async fn move_cursors_left(wrap: &Value) -> Result<Vec<Value>, Condition> {
    let wrap: bool = wrap.clone().try_into()?;
    let state = SessionState::get_state();
    let mut guard = state.lock().await;
    let Some(buffer) = guard.get_current_buffer_mut() else {
        return Err(Condition::error(String::from("Buffer not found")));
    };
    buffer.move_cursors(CursorDirection::Left { wrap }).await;
    Ok(Vec::new())
}

#[bridge(name = "move-cursors-right", lib = "(koru-session)")]
pub async fn move_cursors_right(wrap: &Value) -> Result<Vec<Value>, Condition> {
    let wrap: bool = wrap.clone().try_into()?;
    let state = SessionState::get_state();
    let mut guard = state.lock().await;
    let Some(buffer) = guard.get_current_buffer_mut() else {
        return Err(Condition::error(String::from("Buffer not found")));
    };
    buffer.move_cursors(CursorDirection::Right { wrap }).await;
    Ok(Vec::new())
}