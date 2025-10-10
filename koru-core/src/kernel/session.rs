use std::collections::{HashSet, VecDeque};
use std::error::Error;
use std::sync::{LazyLock, Mutex};
use mlua::Lua;
use crate::kernel::broker::BrokerClient;

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




pub struct Session {
    session_id: SessionId,
    lua: Lua,
    broker_client: BrokerClient,
    client_ids: Vec<usize>,
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
            _ = self.broker_client.recv().await;
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