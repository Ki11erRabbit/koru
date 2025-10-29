mod buffer;

use std::collections::{HashSet, VecDeque};
use std::error::Error;
use std::sync::{LazyLock, Mutex};
use mlua::{AnyUserData, Function, IntoLua, Lua, ObjectLike, Table};
use crate::attr_set::AttrSet;
use crate::kernel::broker::{BrokerClient, GeneralMessage, Message, MessageKind};
use crate::kernel::{lua_api};
use crate::kernel::input::{ControlKey, KeyBuffer, KeyPress, KeyValue};
use crate::kernel::session::buffer::{Buffer, BufferData};
use crate::keybinding::Keybinding;

static ID_MANAGER: LazyLock<Mutex<SessionIdManager>> = LazyLock::new(|| {
    Mutex::new(SessionIdManager::new())
});

#[derive(Copy, Clone, Eq, PartialEq, Hash)]
pub struct SessionId(usize);

impl SessionId {
    pub fn new(id: usize) -> SessionId {
        SessionId(id)
    }
    pub fn get(&self) -> usize {
        self.0
    }
}

pub struct SessionIdManager {
    next_session_id: usize,
    free_ids: VecDeque<usize>,
    active_sessions: HashSet<usize>,
}

impl SessionIdManager {
    fn new() -> Self {
        SessionIdManager {
            next_session_id: 0,
            free_ids: VecDeque::new(),
            active_sessions: HashSet::new(),
        }
    }
    
    fn next_session_id(&mut self) -> SessionId {
        if self.next_session_id == usize::MAX {
            match self.free_ids.pop_front() {
                Some(id) => {
                    self.active_sessions.insert(id);
                    SessionId(id)
                },
                None => {
                    panic!("SessionIdManager free ids exhausted");
                }
            }
        } else {
            let id = self.next_session_id;
            self.next_session_id += 1;
            self.active_sessions.insert(id);
            SessionId(id)
        }
    }
    
    fn remove_session_id(&mut self, session_id: SessionId) {
        self.active_sessions.remove(&session_id.0);
        self.free_ids.push_back(session_id.0);
    }
    
    pub fn get_new_id() -> SessionId {
        let Ok(mut id_manager) = ID_MANAGER.lock() else {
            panic!("SessionIdManager lock poisoned");
        };
        
        id_manager.next_session_id()
    }
    
    pub fn free_id(session_id: SessionId) {
        let Ok(mut id_manager) = ID_MANAGER.lock() else {
            panic!("SessionIdManager lock poisoned");
        };
        id_manager.remove_session_id(session_id);
    }
    
}

pub enum CommandState {
    None,
    EnteringCommand(String),
}


pub struct Session {
    session_id: SessionId,
    lua: Lua,
    broker_client: BrokerClient,
    client_ids: Vec<usize>,
    command_state: CommandState,
    key_buffer: KeyBuffer,
    keybinding: Keybinding<mlua::Function>,
    focused_buffer: mlua::Value,
}

impl Session {
    pub fn new(
        lua: Lua,
        broker_client: BrokerClient,
    ) -> Self {
        let id = SessionIdManager::get_new_id();
        Self { 
            session_id: id,
            lua,
            broker_client,
            client_ids: vec![],
            command_state: CommandState::None,
            key_buffer: KeyBuffer::new(),
            keybinding: Keybinding::new(),
            focused_buffer: mlua::Value::Nil,
        }
    }

    fn set_globals(&self) -> Result<(), Box<dyn Error>> {
        let session_id = self.session_id.0;
        self.lua.globals().set(
            "get_session_id",
            self.lua.create_function(move |_, ()| {
                Ok(session_id)
            })?,
        )?;
        
        self.lua.globals().set(
            "__open_buffers",
            self.lua.create_table()?
        )?;

        self.lua.globals().set(
            "__major_mode",
            self.lua.create_table()?
        )?;

        self.lua.globals().set(
            "set_major_mode",
            self.lua.create_function(|lua, (file_index, mode): (mlua::Value, mlua::Value)| {
                lua.globals().get::<Table>("__major_mode")?.set(file_index, mode)
            })?
        )?;

        self.lua.globals().set(
            "__file_open_hooks",
            self.lua.create_table()?
        )?;

        self.lua.globals().set(
            "add_file_open_hook",
            self.lua.create_function(|lua, (hook_name, mode): (mlua::String, Function)| {
                lua.globals().get::<Table>("__file_open_hooks")?.set(hook_name, mode)
            })?
        )?;

        self.lua.globals().set(
            "__minor_modes",
            self.lua.create_table()?
        )?;

        self.lua.globals().set(
            "add_minor_mode",
            self.lua.create_function(|lua, (file_index, mode_name, mode): (mlua::Value, mlua::String, mlua::Value)| {
                if let Ok(file_modes) = lua.globals().get::<Table>("__minor_modes")?.get::<Table>(file_index.clone()) {
                    file_modes.set(mode_name, mode)?;
                } else {
                    let table = lua.create_table()?;
                    table.set(mode_name, mode)?;
                    lua.globals().get::<Table>("__minor_modes")?.set(file_index, table)?;
                }
                Ok(())
            })?
        )?;
        
        self.create_buffer("**Warnings**", Buffer::new_log())?;
        
        self.lua.globals().set(
            "write_warning",
            self.lua.create_function(|lua, string: mlua::String| {
                let buffer = lua.globals().get::<Table>("__open_buffers")?
                    .get::<AnyUserData>("**Warnings**")?;
                let mut buffer = buffer.borrow_mut::<Buffer>()?;
                
                let string = string.to_str()?;
                
                buffer.manipulate_data(|data| {
                    match data {
                        BufferData::Log(log) => {
                            log.push(string.to_string());
                        }
                        _ => unreachable!("We should only have a log buffer here"),
                    }
                });
                
                Ok(())
            })?
        )?;
        
        self.create_buffer("**Errors**", Buffer::new_log())?;

        self.lua.globals().set(
            "write_error",
            self.lua.create_function(|lua, string: mlua::String| {
                let buffer = lua.globals().get::<Table>("__open_buffers")?
                    .get::<AnyUserData>("**Errors**")?;
                let mut buffer = buffer.borrow_mut::<Buffer>()?;

                let string = string.to_str()?;

                buffer.manipulate_data(|data| {
                    match data {
                        BufferData::Log(log) => {
                            log.push(string.to_string());
                        }
                        _ => unreachable!("We should only have a log buffer here"),
                    }
                });

                Ok(())
            })?
        )?;
        
        let package = self.lua.globals().get::<Table>("package").unwrap();
        let preload = package.get::<Table>("preload").unwrap();

        preload.set(
            "Koru",
            self.lua.create_function(|lua, _:()| {
                lua_api::kernel_mod(&lua)
            })?
        )?;

        self.lua.load(include_str!("../../../lua/textviewmode.lua")).exec()?;
        self.lua.load(include_str!("../../../lua/logviewmode.lua")).exec()?;
        Ok(())
    }
    
    fn write_warning(&self, msg: String) -> Result<(), Box<dyn Error>> {
        let buffer = self.lua.globals().get::<Table>("__open_buffers")?
            .get::<AnyUserData>("**Warnings**")?;
        let mut buffer = buffer.borrow_mut::<Buffer>()?;

        buffer.manipulate_data(move |data| {
            match data {
                BufferData::Log(log) => {
                    log.push(msg);
                }
                _ => unreachable!("We should only have a log buffer here"),
            }
        });
        
        Ok(())
    }
    
    fn write_error(&self, msg: String) -> Result<(), Box<dyn Error>> {
        let buffer = self.lua.globals().get::<Table>("__open_buffers")?
            .get::<AnyUserData>("**Errors**")?;
        let mut buffer = buffer.borrow_mut::<Buffer>()?;

        buffer.manipulate_data(|data| {
            match data {
                BufferData::Log(log) => {
                    log.push(msg);
                }
                _ => unreachable!("We should only have a log buffer here"),
            }
        });
        
        Ok(())
    }
    
    async fn new_client_connection(&mut self, id: usize) -> Result<(), Box<dyn Error>> {
        let ui_attrs = self.lua.globals().get::<Table>("__ui_attrs")?;
        let mut values = Vec::new();
        for pair in ui_attrs.pairs() {
            let (key, value) = pair?;
            match (key, value) {
                (mlua::Value::String(key), mlua::Value::String(value)) => {
                    let key = key.to_str()?.to_string();
                    let value = value.to_str()?.to_string();
                    
                    values.push(AttrSet::new(key, value));
                }
                _ => unreachable!("We should only be able to put strings into ui attributes")
            }
        }
        self.broker_client.send(MessageKind::General(GeneralMessage::SetUiAttrs(values)), id)?;
        
        self.client_ids.push(id);
        
        Ok(())
    }
    
    fn create_buffer(&self, name: &str, buffer: Buffer) -> Result<mlua::Value, Box<dyn Error>> {
        let open_buffers = self.lua.globals().get::<Table>("__open_buffers")?;
        
        open_buffers.set(
            name,
            buffer,
        )?;
        
        Ok(name.into_lua(&self.lua)?)
    }
    
    fn notify_clients(&mut self, msg: MessageKind) {
        let mut dead_clients = Vec::new();
        for (i, client) in self.client_ids.iter().enumerate() {
            match self.broker_client.send(msg.clone(), *client) {
                Ok(_) => {}
                Err(_) => {
                    dead_clients.push(i);
                }
            }
        }
        for client in dead_clients.into_iter().rev() {
            self.client_ids.remove(client);
        }
    }

    fn file_opened_hook(&self, file_name: mlua::Value, file_ext: &str) {
        let file_open_hooks = self.lua.globals().get::<Table>("__file_open_hooks").unwrap();
        for hook in file_open_hooks.pairs::<mlua::String, mlua::Function>() {
            let (_, function) = hook.unwrap();
            match function.call::<()>((file_name.clone(), file_ext.to_string())) {
                Ok(_) => {}
                Err(e) => {
                    self.write_warning(e.to_string()).unwrap()
                }
            }
        }
    }

    fn send_draw(&mut self, buffer_name: mlua::Value) -> Result<(), Box<dyn Error>> {
        
        if buffer_name == mlua::Value::Nil {
            return Ok(());
        }
        
        let open_buffers = self.lua.globals().get::<Table>("__open_buffers")?;
        
        let buffer = open_buffers.get::<AnyUserData>(buffer_name.clone())?;
        let buffer = buffer.borrow::<Buffer>()?;
        
        let styled_file = buffer.styled_file();

        let major_mode = self.lua.globals().get::<Table>("__major_mode")?
            .get::<Table>(buffer_name)?;

        let line_count = styled_file.line_count();

        let styled_file: AnyUserData = major_mode.call_method("modify_line", (styled_file, line_count as i64))?;
        let styled_file = styled_file.take()?;

        self.notify_clients(MessageKind::General(GeneralMessage::Draw(styled_file)));
        Ok(())
    }

    
    pub async fn run(&mut self, session_code: &str, client_id: usize) {

        match self.set_globals() {
            Ok(_) => {}
            Err(e) => {
                eprintln!("{}", e);
                panic!("set_globals failed");
            }
        }
        
        match self.lua.load(session_code).exec_async().await {
            Ok(_) => {}
            Err(e) => {
                self.write_error(e.to_string()).unwrap();
            }
        }
        match self.new_client_connection(client_id).await {
            Ok(_) => {}
            Err(e) => {
                eprintln!("{}", e);
                self.write_error(e.to_string()).unwrap();
            }
        }
        
        loop {
            match self.broker_client.recv() {
                Some(Message { kind: MessageKind::General(GeneralMessage::FlushKeyBuffer), ..}) => {
                    self.key_buffer.clear();
                }
                Some(Message { kind: MessageKind::General(GeneralMessage::KeyEvent(KeyPress { key: KeyValue::CharacterKey('w'), ..})), .. }) => {
                    self.send_draw("**Warnings**".into_lua(&self.lua).unwrap()).unwrap();
                }
                Some(Message { kind: MessageKind::General(GeneralMessage::KeyEvent(KeyPress { key: KeyValue::CharacterKey('e'), ..})), .. }) => {
                    self.send_draw("**Errors**".into_lua(&self.lua).unwrap()).unwrap();
                }
                Some(Message { kind: MessageKind::General(GeneralMessage::KeyEvent(KeyPress { key: KeyValue::ControlKey(ControlKey::Up), ..})), .. }) => {
                    let open_buffers = self.lua.globals().get::<Table>("__open_buffers").unwrap();

                    let buffer = open_buffers.get::<AnyUserData>(self.focused_buffer.clone()).unwrap();
                    
                    let _: () = buffer.call_async_method("cursor_up", ()).await.unwrap();
                    self.send_draw(self.focused_buffer.clone()).unwrap();
                }
                Some(Message { kind: MessageKind::General(GeneralMessage::KeyEvent(KeyPress { key: KeyValue::ControlKey(ControlKey::Down), ..})), .. }) => {
                    let open_buffers = self.lua.globals().get::<Table>("__open_buffers").unwrap();

                    let buffer = open_buffers.get::<AnyUserData>(self.focused_buffer.clone()).unwrap();

                    let _: () = buffer.call_async_method("cursor_down", ()).await.unwrap();
                    self.send_draw(self.focused_buffer.clone()).unwrap();
                }
                Some(Message { kind: MessageKind::General(GeneralMessage::KeyEvent(KeyPress { key: KeyValue::ControlKey(ControlKey::Left), ..})), .. }) => {
                    let open_buffers = self.lua.globals().get::<Table>("__open_buffers").unwrap();

                    let buffer = open_buffers.get::<AnyUserData>(self.focused_buffer.clone()).unwrap();

                    let _: () = buffer.call_async_method("cursor_left", ()).await.unwrap();
                    self.send_draw(self.focused_buffer.clone()).unwrap();
                }
                Some(Message { kind: MessageKind::General(GeneralMessage::KeyEvent(KeyPress { key: KeyValue::ControlKey(ControlKey::Right), ..})), .. }) => {
                    let open_buffers = self.lua.globals().get::<Table>("__open_buffers").unwrap();

                    let buffer = open_buffers.get::<AnyUserData>(self.focused_buffer.clone()).unwrap();

                    let _: () = buffer.call_async_method("cursor_right", ()).await.unwrap();
                    self.send_draw(self.focused_buffer.clone()).unwrap();
                }
                Some(Message { kind: MessageKind::General(GeneralMessage::KeyEvent(KeyPress { key: KeyValue::CharacterKey('j'), ..})), .. }) => {
                    const FILE_NAME: &str = "koru-core/src/kernel.rs";

                    let file = crate::kernel::buffer::TextBufferTable::open(FILE_NAME.to_string()).unwrap();
                    
                    let buffer = Buffer::new_open_file(file);
                    
                    let buffer_name = self.create_buffer(FILE_NAME, buffer).unwrap();
                    self.focused_buffer = buffer_name.clone();
                    
                    
                    self.file_opened_hook(buffer_name.clone(), "rs");
                    match self.send_draw(buffer_name) {
                        Ok(_) => {}
                        Err(e) => {
                            self.write_error(e.to_string()).unwrap();
                        }
                    }
                }
                Some(Message { kind: MessageKind::General(GeneralMessage::KeyEvent(KeyPress { key, ..})), .. }) => {
                    match &mut self.command_state {
                        CommandState::None => {
                            match key {
                                KeyValue::CharacterKey(';') => {
                                    self.command_state = CommandState::EnteringCommand(String::from(": "));
                                    self.notify_clients(MessageKind::General(GeneralMessage::UpdateMessageBar(String::from(": "))));
                                }
                                _ => {}
                            }
                        }
                        CommandState::EnteringCommand(cmd) => {
                            let msg = match key {
                                KeyValue::CharacterKey(c) => {
                                    cmd.push(c);
                                    Some(MessageKind::General(GeneralMessage::UpdateMessageBar(cmd.clone())))
                                }
                                KeyValue::ControlKey(ControlKey::Escape) => {
                                    self.command_state = CommandState::None;
                                    Some(MessageKind::General(GeneralMessage::UpdateMessageBar(String::new())))
                                }
                                KeyValue::ControlKey(ControlKey::Backspace) => {
                                    cmd.pop();
                                    Some(MessageKind::General(GeneralMessage::UpdateMessageBar(cmd.clone())))
                                }
                                KeyValue::ControlKey(ControlKey::Space) => {
                                    cmd.push(' ');
                                    Some(MessageKind::General(GeneralMessage::UpdateMessageBar(cmd.clone())))
                                }
                                _ => {
                                    None
                                }
                            };
                            if let Some(msg) = msg {
                                self.notify_clients(msg);
                            }
                        }
                    }
                }
                Some(message) => {
                    self.notify_clients(MessageKind::General(GeneralMessage::UpdateMessageBar(format!("{:?}", message))));
                    //println!("Received message: {:?}", message);
                }
                _ => {}
            }
        }
        // TODO: add a way to send error to the frontend
    }

    pub async fn run_session(broker_client: BrokerClient, client_id: usize) {
        let lua = Lua::new();
        let mut session = Session::new(lua, broker_client);

        session.run("\
local koru = require \"Koru\"\
local command = require \"Koru.Command\"\
koru.hello()
local command = command('hello', 'prints hello', function()
    print('hello')
end, {})
        ", client_id).await;
    }
}

unsafe impl Send for Session {}
unsafe impl Sync for Session {}


impl Drop for Session {
    fn drop(&mut self) {
        SessionIdManager::free_id(self.session_id);
    }
}