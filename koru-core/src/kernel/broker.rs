use std::collections::VecDeque;
use std::error::Error;
use std::hash::Hash;
use std::sync::LazyLock;
use std::sync::mpsc::{Receiver, Sender};
use guile_rs::{guile_misc_error, guile_wrong_type_arg, Guile, SchemeValue, SmobData, SmobTag};
use guile_rs::scheme_object::{SchemeObject, SchemeSmob};
use crate::attr_set::AttrSet;
use crate::kernel::input::KeyPress;
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

pub static MESSAGE_SMOB_TAG: LazyLock<SmobTag<Message>> = LazyLock::new(|| {
    SmobTag::register("Message")
});

impl SmobData for Message {
    fn heap_size(&self) -> usize {
        0
    }
}

#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub enum MessageKind {
    General(GeneralMessage),
    Broker(BrokerMessage),
}

pub static MESSAGE_KIND_SMOB_TAG: LazyLock<SmobTag<MessageKind>> = LazyLock::new(|| {
    SmobTag::register("MessageKind")
});

impl SmobData for MessageKind {
    fn print(&self) -> String {
        let kind = match self {
            MessageKind::General(GeneralMessage::Command) => "General:Command",
            MessageKind::General(GeneralMessage::Draw(_)) => "General:Draw",
            MessageKind::General(GeneralMessage::FlushKeyBuffer) => "General:FlushKeyBuffer",
            MessageKind::General(GeneralMessage::KeyEvent(_)) => "General:KeyEvent",
            MessageKind::General(GeneralMessage::MouseEvent) => "General:MouseEvent",
            MessageKind::General(GeneralMessage::SetColorDef(_)) => "General:SetColorDef",
            MessageKind::General(GeneralMessage::SetUiAttrs(_)) => "General:SetUiAttrs",
            MessageKind::General(GeneralMessage::UpdateMessageBar(_)) => "General:UpdateMessageBar",
            MessageKind::Broker(BrokerMessage::Shutdown) => "Broker:Shutdown",
            MessageKind::Broker(BrokerMessage::ConnectedToSession(_)) => "Broker:ConnectedToSession",
            MessageKind::Broker(BrokerMessage::ConnectToSession) => "Broker:ConnectToSession",
            MessageKind::Broker(BrokerMessage::CreateClient) => "Broker:CreateClient",
            MessageKind::Broker(BrokerMessage::CreateClientResponse(_)) => "Broker:CreateClientResponse",
        };
        format!("#<MessageKind:{kind}>")
    }

    fn heap_size(&self) -> usize {
        0
    }

    fn eq(&self, other: SchemeObject) -> bool {
        let Some(other) = other.cast_smob(MESSAGE_KIND_SMOB_TAG.clone()) else {
            return false;
        };
        *self == *other.borrow()
    }
}

#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub enum GeneralMessage {
    KeyEvent(KeyPress),
    MouseEvent,
    Command,
    Draw(StyledFile),
    SetColorDef(ColorDefinition),
    UpdateMessageBar(String),
    FlushKeyBuffer,
    SetUiAttrs(Vec<AttrSet>),
}

#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub enum BrokerMessage {
    /// This tells the broker to shut down the connection to this client
    Shutdown,
    CreateClient,
    CreateClientResponse(BrokerClient),
    ConnectToSession,
    ConnectedToSession(usize),
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
        self.sender.send(msg)?;
        Ok(())
    }

    pub fn send_response(&mut self, message: MessageKind, mail: Message) -> Result<(), Box<dyn Error>> {
        let msg = mail.make_response(message);
        self.sender.send(msg)?;
        Ok(())
    }
    
    pub fn recv(&mut self) -> Option<Message> {
        self.receiver.recv().ok()
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
        let (_, receiver) = std::sync::mpsc::channel();
        Self::new(self.client_id, self.sender.clone(), receiver)
    }
}


pub static BROKER_CLIENT_SMOB_TAG: LazyLock<SmobTag<BrokerClient>> = LazyLock::new(|| {
    SmobTag::register("BrokerClient")
});

impl SmobData for BrokerClient {
    fn print(&self) -> String {
        format!("#<BrokerClient:{}>", self.client_id)
    }
    fn heap_size(&self) -> usize {
        0
    }
}


pub struct Broker {
    clients: Vec<Option<Sender<Message>>>,
    free_clients: VecDeque<usize>,
    receiver: Receiver<Message>,
    sender: Sender<Message>
}

impl Broker {
    pub fn new() -> Broker {
        let (sender, receiver) = std::sync::mpsc::channel();
        Broker {
            clients: Vec::new(),
            free_clients: VecDeque::new(),
            receiver,
            sender,
        }
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
        let (sender, receiver) = std::sync::mpsc::channel();
        
        self.clients[id] = Some(sender);

        BrokerClient::new(id, self.sender.clone(), receiver)
    }
    
    pub async fn run_broker(&mut self) -> Result<(), Box<dyn Error>> {
        loop {
            let message = self.receiver.recv()?;
            
            match message.kind {
                MessageKind::Broker(BrokerMessage::Shutdown) => {
                    self.free_client(message.source);
                }
                MessageKind::General(_) => {
                    self.send(message)?;
                }
                MessageKind::Broker(BrokerMessage::CreateClient) => {
                    let client = self.create_client();
                    let response = MessageKind::Broker(BrokerMessage::CreateClientResponse(client));
                    self.send_response(message, response)?;
                }
                MessageKind::Broker(BrokerMessage::ConnectToSession) => {
                    self.create_editor_session(message)?;
                }
                _ => {}
            }
        }
        Ok(())
    }
    
    fn send_response(&mut self, message: Message, response: MessageKind) -> Result<(), Box<dyn Error>> {
        let message = message.make_response(response);
        self.send(message)?;
        Ok(())
    }
    
    fn send(&mut self, message: Message) -> Result<(), Box<dyn Error>> {
        match &mut self.clients[message.destination] {
            Some(client) => {
                client.send(message)?;
            }
            None => {}
        }
        Ok(())
    }
    
    fn create_editor_session(&mut self, message: Message) -> Result<(), Box<dyn Error>> {
        let session_client = self.create_client();
        let response = MessageKind::Broker(BrokerMessage::ConnectedToSession(session_client.id()));
        tokio::spawn(Session::run_session(session_client, message.source));
        self.send_response(message, response)
    }
}



pub fn broker_module() {
    
}