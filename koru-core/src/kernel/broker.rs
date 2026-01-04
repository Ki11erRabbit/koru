use std::collections::VecDeque;
use std::error::Error;
use std::hash::Hash;
use std::panic::{catch_unwind, AssertUnwindSafe};
use futures::FutureExt;
use log::error;
use tokio::runtime::Handle;
use tokio::sync::mpsc::{Receiver, Sender};
use crate::attr_set::AttrSet;
use crate::kernel::input::KeyPress;
use crate::kernel::scheme_api::session::SessionState;
use crate::kernel::session::Session;
use crate::styled_text::{ColorDefinition, StyledFile};

#[derive(Debug, Clone, Eq, Hash)]
pub struct Message {
    destination: usize,
    source: usize,
    pub kind: MessageKind,
}

impl Message {
    pub fn new(destination: usize, source: usize, kind: MessageKind) -> Self {
        Self { destination, source, kind }
    }
    pub fn make_response(self, kind: MessageKind) -> Self {
        Self { destination: self.source, source: self.destination, kind }
    }
}

impl PartialEq for Message {
    fn eq(&self, _: &Self) -> bool {
        true
    }
}


#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub enum MessageKind {
    General(GeneralMessage),
    Broker(BrokerMessage),
    BackEnd(BackendMessage)
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub enum GeneralMessage {
    KeyEvent(KeyPress),
    MouseEvent,
    Command,
    Draw(StyledFile),
    SetColorDef(ColorDefinition),
    UpdateMessageBar(String),
    FlushKeyBuffer,
    SetUiAttrs(Vec<AttrSet>),
    RequestMainCursor,
    MainCursorPosition(usize, usize),
    ShowCommandBar,
    HideCommandBar,
    UpdateCommandBar(String),
}

impl Hash for GeneralMessage {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        
    }
}

#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub enum BrokerMessage {
    /// This tells the broker to shut down the connection to this client
    Shutdown,
    CreateClient,
    CreateClientResponse(BrokerClient),
    ConnectToSession,
    ConnectedToSession(usize),
    Crash,
}

#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub enum BackendMessage {
    ShowCommandBar,
    HideCommandBar,
    UpdateCommandBar(String),
}


#[derive(Debug)]
pub struct BrokerClient {
    client_id: usize,
    sender: Sender<Message>,
    receiver: Receiver<Message>,
}

impl BrokerClient {
    pub fn new(client_id: usize, sender: Sender<Message>, receiver: Receiver<Message>) -> Self {
        Self { client_id, sender, receiver }
    }
    
    pub fn send(&mut self, message: MessageKind, destination: usize) -> Result<(), Box<dyn Error>> {
        let msg = Message::new(destination, self.client_id, message);
        self.sender.blocking_send(msg)?;
        Ok(())
    }

    pub fn send_response(&mut self, message: MessageKind, mail: Message) -> Result<(), Box<dyn Error>> {
        let msg = mail.make_response(message);
        self.sender.blocking_send(msg)?;
        Ok(())
    }
    
    pub fn recv(&mut self) -> Option<Message> {
        self.receiver.blocking_recv()
    }

    pub async fn send_async(&mut self, message: MessageKind, destination: usize) -> Result<(), Box<dyn Error>> {
        let msg = Message::new(destination, self.client_id, message);
        self.sender.send(msg).await?;
        Ok(())
    }

    pub async fn send_response_async(&mut self, message: MessageKind, mail: Message) -> Result<(), Box<dyn Error>> {
        let msg = mail.make_response(message);
        self.sender.send(msg).await?;
        Ok(())
    }

    pub async fn recv_async(&mut self) -> Option<Message> {
        self.receiver.recv().await
    }
    
    pub fn id(&self) -> usize {
        self.client_id
    }
}

impl PartialEq for BrokerClient {
    fn eq(&self, other: &Self) -> bool {
        self.id() == other.id()
    }
}

impl Eq for BrokerClient {}

impl Hash for BrokerClient {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.id().hash(state);
    }
}

impl Clone for BrokerClient {
    fn clone(&self) -> Self {
        let (_, receiver) = tokio::sync::mpsc::channel(1);
        Self::new(self.client_id, self.sender.clone(), receiver)
    }
}


pub struct Broker {
    clients: Vec<Option<Sender<Message>>>,
    free_clients: VecDeque<usize>,
    receiver: Receiver<Message>,
    sender: Sender<Message>,
    client_connector_client: usize,
    backend_client: usize,
}

impl Broker {
    pub async fn new() -> (Broker, BrokerClient) {
        let (sender, receiver) = tokio::sync::mpsc::channel(100);
        let mut broker = Broker {
            clients: Vec::new(),
            free_clients: VecDeque::new(),
            receiver,
            sender,
            client_connector_client: 0,
            backend_client: 0,
        };
        let client = broker.create_client();
        broker.client_connector_client = client.client_id;
        broker.create_backend_client().await;
        (broker, client)
    }
    
    fn get_next_client_id(&mut self) -> usize {
        if let Some(client_id) = self.free_clients.pop_front() {
            return client_id;
        } 
        let id = self.clients.len();
        self.clients.push(None);
        id
    }
    
    fn free_client(&mut self, id: usize) {
        self.free_clients.push_back(id);
        self.clients[id] = None;
    }
    
    pub fn create_client(&mut self) -> BrokerClient {
        let id = self.get_next_client_id();
        let (sender, receiver) = tokio::sync::mpsc::channel(100);
        
        self.clients[id] = Some(sender);

        BrokerClient::new(id, self.sender.clone(), receiver)
    }

    async fn create_backend_client(&mut self) {
        let client = self.create_client();
        self.backend_client = client.client_id;
        SessionState::set_broker_client(client).await;
    }
    
    pub async fn run_broker(&mut self) -> Result<(), Box<dyn Error>> {
        loop {
            let message = match self.receiver.recv().await {
                Some(message) => message,
                None => {
                    error!("All Clients dropped");
                    return Ok(())
                },
            };
            
            match message.kind {
                MessageKind::Broker(BrokerMessage::Shutdown) => {
                    self.free_client(message.source);
                    let mut client_counts = 0;
                    for client in &mut self.clients {
                        if client.is_some() {
                            client_counts += 1;
                        }
                    }
                    if client_counts == 0 {
                        return Ok(());
                    } else if client_counts == 2
                        && self.clients[self.client_connector_client].is_some()
                        && self.clients[self.backend_client].is_some() {
                        let message = Message::new(self.client_connector_client, 0, MessageKind::Broker(BrokerMessage::Shutdown));
                        self.sender.send(message).await?;
                        return Ok(());
                    }
                }
                MessageKind::General(_) => {
                    self.send(message).await?;
                }
                MessageKind::Broker(BrokerMessage::CreateClient) => {
                    let client = self.create_client();
                    let response = MessageKind::Broker(BrokerMessage::CreateClientResponse(client));
                    self.send_response(message, response).await?;
                }
                MessageKind::Broker(BrokerMessage::ConnectToSession) => {
                    self.create_editor_session(message).await?;
                }
                MessageKind::BackEnd(_) => {
                    self.send(message).await?;
                }
                MessageKind::Broker(BrokerMessage::Crash) => {
                    for client in &mut self.clients.iter() {
                        if let Some(sender) = client {
                            sender.send(message.clone()).await?;
                        }
                    }
                }
                _ => {}
            }
        }
    }
    
    async fn send_response(&mut self, message: Message, response: MessageKind) -> Result<(), Box<dyn Error>> {
        let message = message.make_response(response);
        self.send(message).await?;
        Ok(())
    }
    
    async fn send(&mut self, message: Message) -> Result<(), Box<dyn Error>> {
        match &mut self.clients[message.destination] {
            Some(client) => {
                client.send(message).await?;
            }
            None => {}
        }
        Ok(())
    }
    
    async fn create_editor_session(&mut self, message: Message) -> Result<(), Box<dyn Error>> {
        let session_client = self.create_client();
        let response = MessageKind::Broker(BrokerMessage::ConnectedToSession(session_client.id()));
        let source = message.source;
        tokio::spawn(async move {
            // We catch the unwind so that we can report to the user that the editor crashed;
            let result = AssertUnwindSafe(Session::run_session(session_client, source)).catch_unwind().await;
            match result {
                Ok(_) => {}
                Err(_) => {
                    SessionState::send_message(MessageKind::Broker(BrokerMessage::Crash)).await
                        .expect("Unable to send message back to frontend");
                }
            }
        });
        self.send_response(message, response).await
    }
}
