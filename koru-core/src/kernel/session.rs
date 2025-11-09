use scheme_rs::cps::Compile;
use std::error::Error;
use std::ffi::OsStr;
use std::path::{Path, PathBuf};
use scheme_rs::ast::DefinitionBody;
use scheme_rs::env::{Environment, Var};
use scheme_rs::num::Number;
use scheme_rs::proc::Procedure;
use scheme_rs::registry::Library;
use scheme_rs::runtime::Runtime;
use scheme_rs::syntax::{Identifier, Span, Syntax};
use scheme_rs::value::Value;
use crate::kernel;
use crate::kernel::broker::{BrokerClient, GeneralMessage, Message, MessageKind};
use crate::kernel::buffer::TextBufferTable;
use crate::kernel::input::{ControlKey, KeyBuffer, KeyPress, KeyValue};
use crate::kernel::scheme_api::session::SessionState;
use crate::keybinding::Keybinding;

pub enum CommandState {
    None,
    EnteringCommand(String),
}


pub struct Session {
    runtime: Runtime,
    env: Environment,
    broker_client: BrokerClient,
    client_ids: Vec<usize>,
    command_state: CommandState,
    key_buffer: KeyBuffer,
    keybinding: Keybinding<Procedure>,
}

impl Session {
    pub async fn new(
        broker_client: BrokerClient,
    ) -> Self {
        let runtime = kernel::SCHEME_RUNTIME.lock().await.take().unwrap();

        let prog = Library::new_program(&runtime, &Path::new("scheme/text-edit-mode.scm"));
        let env = Environment::Top(prog);

        let sexprs = Syntax::from_str(include_str!("../../../scheme/text-edit-mode.scm"), Some("text-edit-mode.scm")).unwrap();
        let span = Span::default();
        let base = DefinitionBody::parse_lib_body(
            &runtime,
            &sexprs,
            &env,
            &span,
        ).await.unwrap();

        let compiled = base.compile_top_level();
        let proc = runtime.compile_expr(compiled).await;
        proc.call(&[]).await.unwrap();

        Self {
            runtime,
            env,
            broker_client,
            client_ids: vec![],
            command_state: CommandState::None,
            key_buffer: KeyBuffer::new(),
            keybinding: Keybinding::new(),
        }
    }

    async fn set_globals(&self) -> Result<(), Box<dyn Error>> {
        Ok(())
    }
    
    fn write_warning(&self, _msg: String) -> Result<(), Box<dyn Error>> {
        todo!()
    }
    
    fn write_error(&self, msg: String) -> Result<(), Box<dyn Error>> {
        println!("{}", msg);
        todo!()
    }
    
    async fn new_client_connection(&mut self, id: usize) -> Result<(), Box<dyn Error>> {
        //todo!();
        /*let ui_attrs = self.lua.globals().get::<Table>("__ui_attrs")?;
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
        self.broker_client.send_async(MessageKind::General(GeneralMessage::SetUiAttrs(values)), id).await?;*/
        
        self.client_ids.push(id);
        
        Ok(())
    }
    
    async fn create_buffer(&self, name: &str) -> Result<String, Box<dyn Error>> {
        let out = name.to_string();
        let handle = TextBufferTable::open(name.to_string()).await?;

        {
            let state = SessionState::get_state();
            let mut guard = state.lock().await;
            guard.add_buffer(name, handle);
        }
        let path = PathBuf::from(name);
        let ext = path.extension().unwrap_or(&OsStr::new("")).to_str().unwrap();
        self.file_opened_hook(name, ext).await;
        Ok(out)
    }
    
    async fn notify_clients(&mut self, msg: MessageKind) {
        let mut dead_clients = Vec::new();
        for (i, client) in self.client_ids.iter().enumerate() {
            match self.broker_client.send_async(msg.clone(), *client).await {
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

    async fn file_opened_hook(&self, file_name: &str, file_ext: &str) {

        let hooks = {
            let state = SessionState::get_state();
            state.lock().await.get_hooks().clone()
        };
        let args = &[Value::from(file_name.to_string()), Value::from(file_ext.to_string())];

        hooks.lock().await.execute_hook("file-open", args).await.unwrap();

        /*let file_open_hooks = self.lua.globals().get::<Table>("__file_open_hooks").unwrap();
        for hook in file_open_hooks.pairs::<mlua::String, mlua::Function>() {
            let (_, function) = hook.unwrap();
            match function.call::<()>((file_name.clone(), file_ext.to_string())) {
                Ok(_) => {}
                Err(e) => {
                    self.write_warning(e.to_string()).unwrap()
                }
            }
        }*/
    }

    async fn send_draw(&mut self, buffer_name: &str) -> Result<(), Box<dyn Error>> {

        let buffer = {
            let state = SessionState::get_state();
            let mut guard = state.lock().await;
            guard.get_buffers().get(buffer_name).unwrap().clone()
        };

        //let styled_file = buffer.get_styled_text().await;
        //self.notify_clients(MessageKind::General(GeneralMessage::Draw(styled_file))).await;
        Ok(())
    }

    
    pub async fn run(&mut self, client_id: usize) {

        match self.set_globals().await {
            Ok(_) => {}
            Err(e) => {
                eprintln!("{}", e);
                panic!("set_globals failed");
            }
        }
        
        /*match self.lua.load(session_code).exec_async().await {
            Ok(_) => {}
            Err(e) => {
                self.write_error(e.to_string()).unwrap();
            }
        }*/
        match self.new_client_connection(client_id).await {
            Ok(_) => {}
            Err(e) => {
                eprintln!("{}", e);
                self.write_error(e.to_string()).unwrap();
            }
        }
        
        loop {
            let message = self.broker_client.recv_async().await;
            match message {
                Some(Message { kind: MessageKind::General(GeneralMessage::FlushKeyBuffer), ..}) => {
                    self.key_buffer.clear();
                }
                Some(Message { kind: MessageKind::General(GeneralMessage::KeyEvent(KeyPress { key: KeyValue::ControlKey(ControlKey::Up), ..})), .. }) => {
                    let move_cursor: Var = self.env.fetch_var(&Identifier::new("move-cursor-up")).await.unwrap().unwrap();

                    let function: Procedure = match move_cursor {
                        Var::Global(value) => value.value().read().clone().try_into().unwrap(),
                        Var::Local(_) => unimplemented!("fetching var from local"),
                    };
                    function.call(&[Value::from(Number::FixedInteger(0))]).await.unwrap();
                    
                    let focused_buffer = {
                        let state = SessionState::get_state();
                        let guard = state.lock().await;
                        guard.current_focused_buffer().unwrap().clone()
                    };
                    self.send_draw(&focused_buffer).await.unwrap();
                }
                Some(Message { kind: MessageKind::General(GeneralMessage::KeyEvent(KeyPress { key: KeyValue::ControlKey(ControlKey::Down), ..})), .. }) => {
                    let move_cursor: Var = self.env.fetch_var(&Identifier::new("move-cursor-down")).await.unwrap().unwrap();

                    let function: Procedure = match move_cursor {
                        Var::Global(value) => value.value().read().clone().try_into().unwrap(),
                        Var::Local(_) => unimplemented!("fetching var from local"),
                    };
                    function.call(&[Value::from(Number::FixedInteger(0))]).await.unwrap();
                    let focused_buffer = {
                        let state = SessionState::get_state();
                        let guard = state.lock().await;
                        guard.current_focused_buffer().unwrap().clone()
                    };
                    self.send_draw(&focused_buffer).await.unwrap();
                }
                Some(Message { kind: MessageKind::General(GeneralMessage::KeyEvent(KeyPress { key: KeyValue::ControlKey(ControlKey::Left), ..})), .. }) => {
                    let move_cursor: Var = self.env.fetch_var(&Identifier::new("move-cursor-left")).await.unwrap().unwrap();

                    let function: Procedure = match move_cursor {
                        Var::Global(value) => value.value().read().clone().try_into().unwrap(),
                        Var::Local(_) => unimplemented!("fetching var from local"),
                    };
                    function.call(&[Value::from(Number::FixedInteger(0)), Value::from(false)]).await.unwrap();
                    let focused_buffer = {
                        let state = SessionState::get_state();
                        let guard = state.lock().await;
                        guard.current_focused_buffer().unwrap().clone()
                    };
                    self.send_draw(&focused_buffer).await.unwrap();
                }
                Some(Message { kind: MessageKind::General(GeneralMessage::KeyEvent(KeyPress { key: KeyValue::ControlKey(ControlKey::Right), ..})), .. }) => {
                    let move_cursor: Var = self.env.fetch_var(&Identifier::new("move-cursor-right")).await.unwrap().unwrap();

                    let function: Procedure = match move_cursor {
                        Var::Global(value) => value.value().read().clone().try_into().unwrap(),
                        Var::Local(_) => unimplemented!("fetching var from local"),
                    };
                    function.call(&[Value::from(Number::FixedInteger(0)), Value::from(false)]).await.unwrap();
                    let focused_buffer = {
                        let state = SessionState::get_state();
                        let guard = state.lock().await;
                        guard.current_focused_buffer().unwrap().clone()
                    };
                    self.send_draw(&focused_buffer).await.unwrap();
                }
                Some(Message { kind: MessageKind::General(GeneralMessage::KeyEvent(KeyPress { key: KeyValue::CharacterKey('m'), ..})), .. }) => {
                    println!("placing mark");
                    let place_mark: Var = self.env.fetch_var(&Identifier::new("place-cursor-mark")).await.unwrap().unwrap();

                    let function: Procedure = match place_mark {
                        Var::Global(value) => value.value().read().clone().try_into().unwrap(),
                        Var::Local(_) => unimplemented!("fetching var from local"),
                    };
                    function.call(&[Value::from(Number::FixedInteger(0))]).await.unwrap();
                    let focused_buffer = {
                        let state = SessionState::get_state();
                        let guard = state.lock().await;
                        guard.current_focused_buffer().unwrap().clone()
                    };
                    self.send_draw(&focused_buffer).await.unwrap();
                }
                Some(Message { kind: MessageKind::General(GeneralMessage::KeyEvent(KeyPress { key: KeyValue::CharacterKey('r'), ..})), .. }) => {
                    println!("removing mark");
                    let remove_mark: Var = self.env.fetch_var(&Identifier::new("remove-cursor-mark")).await.unwrap().unwrap();

                    let function: Procedure = match remove_mark {
                        Var::Global(value) => value.value().read().clone().try_into().unwrap(),
                        Var::Local(_) => unimplemented!("fetching var from local"),
                    };
                    function.call(&[Value::from(Number::FixedInteger(0))]).await.unwrap();
                    let focused_buffer = {
                        let state = SessionState::get_state();
                        let guard = state.lock().await;
                        guard.current_focused_buffer().unwrap().clone()
                    };
                    self.send_draw(&focused_buffer).await.unwrap();
                }
                Some(Message { kind: MessageKind::General(GeneralMessage::KeyEvent(KeyPress { key: KeyValue::CharacterKey('j'), ..})), .. }) => {
                    const FILE_NAME: &str = "koru-core/src/kernel/session.rs";
                    
                    let buffer_name = self.create_buffer(FILE_NAME).await.unwrap();
                    let focused_buffer = {
                        let state = SessionState::get_state();
                        let mut guard = state.lock().await;
                        guard.set_current_buffer(buffer_name.to_string());
                        guard.current_focused_buffer().unwrap().clone()
                    };
                    self.send_draw(&focused_buffer).await.unwrap();
                    match self.send_draw(FILE_NAME).await {
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
    
    fn get_runtime() -> Runtime {
        Runtime::new()
    }

    pub async fn run_session(broker_client: BrokerClient, client_id: usize) {
        let mut session = Session::new(broker_client).await;

        session.run(client_id).await;
    }
}

unsafe impl Send for Session {}
unsafe impl Sync for Session {}
