use std::collections::{HashSet, VecDeque};
use std::error::Error;
use std::sync::{LazyLock, Mutex};
use mlua::Lua;
use crate::kernel::broker::{BrokerClient, GeneralMessage, Message, MessageKind};
use crate::kernel::files;
use crate::kernel::files::{OpenFileHandle, OpenFileTable};
use crate::kernel::input::{ControlKey, KeyPress, KeyValue};
use crate::styled_text::StyledFile;

static ID_MANAGER: LazyLock<Mutex<SessionIdManager>> = LazyLock::new(|| {
    Mutex::new(SessionIdManager::new())
});

#[derive(Copy, Clone)]
pub struct SessionId(usize);

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
    open_file: Vec<OpenFileHandle>,
    command_state: CommandState,
}

impl Session {
    pub fn new(
        lua: Lua,
        broker_client: BrokerClient,
        client_id: usize,
    ) -> Self {
        let id = SessionIdManager::get_new_id();
        Self { 
            session_id: id,
            lua,
            broker_client,
            client_ids: vec![client_id],
            open_file: Vec::new(),
            command_state: CommandState::None,
        }
    }
    
    async fn notify_clients(&mut self, msg: MessageKind) {
        let mut dead_clients = Vec::new();
        for (i, client) in self.client_ids.iter().enumerate() {
            match self.broker_client.send(msg.clone(), *client).await {
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
    
    pub async fn run(&mut self, session_code: &str) {
        let session_id = self.session_id.0;
        self.lua.globals().set(
            "get_session_id",
            self.lua.create_function(move |_, ()| {
                Ok(session_id)
            }).unwrap(),
        ).unwrap();
        
        self.lua.load(session_code).exec_async().await.unwrap();
        loop {
            match self.broker_client.recv().await {
                Some(Message { kind: MessageKind::General(GeneralMessage::KeyEvent(KeyPress { key: KeyValue::CharacterKey('j'), ..})), .. }) => {
                    let file = files::open_file("koru-core/src/kernel.rs").await.unwrap();
                    let text = file.get_text().await;
                    self.open_file.push(file);
                    let styled_file = StyledFile::from(text);
                    self.notify_clients(MessageKind::General(GeneralMessage::Draw(styled_file))).await;
                }
                Some(Message { kind: MessageKind::General(GeneralMessage::KeyEvent(KeyPress { key, ..})), .. }) => {
                    match &mut self.command_state {
                        CommandState::None => {
                            match key {
                                KeyValue::CharacterKey(';') => {
                                    self.command_state = CommandState::EnteringCommand(String::from(": "));
                                    self.notify_clients(MessageKind::General(GeneralMessage::UpdateMessageBar(String::from(": ")))).await;
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
                                self.notify_clients(msg).await;
                            }
                        }
                    }
                }
                Some(message) => {
                    self.notify_clients(MessageKind::General(GeneralMessage::UpdateMessageBar(format!("{:?}", message)))).await;
                    //println!("Received message: {:?}", message);
                }
                _ => {}
            }
        }
        // TODO: add a way to send error to the frontend
    }

    pub async fn run_session(broker_client: BrokerClient, client_id: usize) {
        let lua = Lua::new();
        let mut session = Session::new(lua, broker_client, client_id);

        session.run("print('Hello, World!')").await;
    }
}

unsafe impl Send for Session {}
unsafe impl Sync for Session {}


impl Drop for Session {
    fn drop(&mut self) {
        SessionIdManager::free_id(self.session_id);
    }
}