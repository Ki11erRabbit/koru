use std::collections::VecDeque;
use std::error::Error;
use std::hash::Hash;
use std::mem::ManuallyDrop;
use std::sync::LazyLock;
use std::sync::mpsc::{Receiver, Sender};
use guile_rs::{Guile, SchemeValue, Smob, SmobData, SmobDrop, SmobEqual, SmobPrint, SmobSize};
use guile_rs::scheme_object::SchemeObject;
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

pub static MESSAGE_SMOB_TAG: LazyLock<Smob<Message>> = LazyLock::new(|| {
    Smob::register("Message")
});

impl SmobData for Message {}
impl SmobEqual for Message {}
impl SmobSize for Message {}
impl SmobPrint for Message {
    fn print(&self) -> String {
        String::from("#<Message>")
    }
}
impl SmobDrop for Message {
    fn drop(&mut self) -> usize {
        let heap_size = self.heap_size();
        let _ = std::mem::replace(&mut self.kind, MessageKind::Blank);
        heap_size
    }

    fn heap_size(&self) -> usize {
        self.kind.heap_size()
    }
}

#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub enum MessageKind {
    Blank,
    General(GeneralMessage),
    Broker(BrokerMessage),
}

pub static MESSAGE_KIND_SMOB_TAG: LazyLock<Smob<MessageKind>> = LazyLock::new(|| {
    Smob::register("MessageKind")
});

impl SmobData for MessageKind {}
impl SmobEqual for MessageKind {}
impl SmobSize for MessageKind {}
impl SmobPrint for MessageKind {
    fn print(&self) -> String {
        String::from("#<MessageKind>")
    }
}
impl SmobDrop for MessageKind {
    fn drop(&mut self) -> usize {
        let heap_size = self.heap_size();
        let _ = std::mem::replace(self, MessageKind::Blank);
        heap_size
    }

    fn heap_size(&self) -> usize {
        match self {
            MessageKind::General(GeneralMessage::Draw(styled_file)) => {
                styled_file.lines().iter().map(|line| line.len()).sum()
            }
            MessageKind::General(GeneralMessage::SetUiAttrs(attrs)) => {
                size_of::<AttrSet>() * attrs.capacity()
            }
            MessageKind::General(GeneralMessage::UpdateMessageBar(string)) => {
                string.capacity()
            }
            _ => 0
        }
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
pub struct BrokerClientInternal {
    client_id: usize,
    sender: Sender<Message>,
    receiver: Receiver<Message>,
}

#[derive(Debug)]
pub struct BrokerClient {
    internal: ManuallyDrop<BrokerClientInternal>,
}

impl BrokerClient {
    pub fn new(client_id: usize, sender: Sender<Message>, receiver: Receiver<Message>) -> Self {
        Self {
            internal: ManuallyDrop::new(
                BrokerClientInternal
                { client_id, sender, receiver })
        }
    }
    
    pub fn send(&mut self, message: MessageKind, destination: usize) -> Result<(), Box<dyn Error>> {
        let msg = Message::new(destination, self.internal.client_id, message);
        self.internal.sender.send(msg)?;
        Ok(())
    }

    pub fn send_response(&mut self, message: MessageKind, mail: Message) -> Result<(), Box<dyn Error>> {
        let msg = mail.make_response(message);
        self.internal.sender.send(msg)?;
        Ok(())
    }
    
    pub fn recv(&mut self) -> Option<Message> {
        self.internal.receiver.recv().ok()
    }
    
    pub fn id(&self) -> usize {
        self.internal.client_id
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
        Self::new(self.internal.client_id, self.internal.sender.clone(), receiver)
    }
}

impl Drop for BrokerClient {
    fn drop(&mut self) {
        unsafe {
            ManuallyDrop::drop(&mut self.internal);
        }
    }
}

pub static BROKER_CLIENT_SMOB_TAG: LazyLock<Smob<BrokerClient>> = LazyLock::new(|| {
    Smob::register("BrokerClient")
});

impl SmobData for BrokerClient {}
impl SmobSize for BrokerClient {}
impl SmobDrop for BrokerClient {
    fn drop(&mut self) -> usize {
        unsafe {
            ManuallyDrop::drop(&mut self.internal);
        }
        self.heap_size()
    }
    fn heap_size(&self) -> usize {
        0
    }
}
impl SmobEqual for BrokerClient {}
impl SmobPrint for BrokerClient {
    fn print(&self) -> String {
        format!("#<BrokerClient:{}>", self.internal.client_id)
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
            let Some(message) = self.receiver.recv()? else {
                break;
            };
            
            match message.kind {
                MessageKind::Broker(BrokerMessage::Shutdown) => {
                    self.free_client(message.source);
                }
                MessageKind::General(_) => {
                    self.send(message).await?;
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

extern "C" fn send_message(client: SchemeValue, message: SchemeValue, destination: SchemeValue) -> SchemeValue {
    let Some(mut client) = SchemeObject::new(client).cast_smob(BROKER_CLIENT_SMOB_TAG.clone()) else {
        Guile::wrong_type_arg(b"send-message\0", 1, client);
    };
    let Some(message) = SchemeObject::new(message).cast_smob(MESSAGE_KIND_SMOB_TAG.clone()) else {
        Guile::wrong_type_arg(b"send-message\0", 2, client);
    };
    let Some(destination) = SchemeObject::new(destination).cast_number() else {
        Guile::wrong_type_arg(b"send-message\0", 3, client);
    };
    let destination = destination.as_u64() as usize;
    match client.send((*message).clone(), destination) {
        Ok(_) => {}
        Err(e) => {
            panic!("{}", e);
        }
    }
    SchemeObject::undefined().into()
}

extern "C" fn recv_message(client: SchemeValue, ) -> SchemeValue {
    let Some(mut client) = SchemeObject::new(client).cast_smob(BROKER_CLIENT_SMOB_TAG.clone()) else {
        Guile::wrong_type_arg(b"send-message\0", 1, client);
    };
    
    match client.recv() {
        Some(message) => {
            return MESSAGE_SMOB_TAG.make(message).into();
        }
        None => {
            panic!("sender died");
        }
    }
}

extern "C" fn send_response(client: SchemeValue, message: SchemeValue, mail: SchemeValue) -> SchemeValue {
    let Some(mut client) = SchemeObject::new(client).cast_smob(BROKER_CLIENT_SMOB_TAG.clone()) else {
        Guile::wrong_type_arg(b"send-message\0", 1, client);
    };
    let Some(message) = SchemeObject::new(message).cast_smob(MESSAGE_KIND_SMOB_TAG.clone()) else {
        Guile::wrong_type_arg(b"send-message\0", 2, client);
    };
    let Some(mail) = SchemeObject::new(mail).cast_smob(MESSAGE_SMOB_TAG.clone()) else {
        Guile::wrong_type_arg(b"send-message\0", 3, client);
    };
    match client.send_response((*message).clone(), (*mail).clone()) {
        Ok(_) => {}
        Err(e) => {
            panic!("{}", e);
        }
    }
    SchemeObject::undefined().into()
}

pub fn broker_module() {
    
}