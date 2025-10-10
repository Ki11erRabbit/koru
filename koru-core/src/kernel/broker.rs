use std::collections::VecDeque;
use std::error::Error;
use tokio::sync::mpsc::{Receiver, Sender};
use crate::kernel::input::KeyPress;
use crate::kernel::session::Session;

#[derive(Debug)]
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

#[derive(Debug)]
pub enum MessageKind {
    General(GeneralMessage),
    Broker(BrokerMessage),
}

#[derive(Debug)]
pub enum GeneralMessage {
    KeyEvent(KeyPress),
    MouseEvent,
    Command,
}

#[derive(Debug)]
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
    
    pub async fn send(&mut self, message: MessageKind, destination: usize) -> Result<(), Box<dyn Error>> {
        let msg = Message::new(destination, self.client_id, message);
        self.sender.send(msg).await?;
        Ok(())
    }

    pub async fn send_response(&mut self, message: MessageKind, mail: Message) -> Result<(), Box<dyn Error>> {
        let msg = mail.make_response(message);
        self.sender.send(msg).await?;
        Ok(())
    }
    
    pub async fn recv(&mut self) -> Option<Message> {
        self.receiver.recv().await
    }
    
    pub fn id(&self) -> usize {
        self.client_id
    }
}

impl Clone for BrokerClient {
    fn clone(&self) -> Self {
        let (_, receiver) = tokio::sync::mpsc::channel(1);
        Self { 
            client_id: self.client_id,
            sender: self.sender.clone(),
            receiver,
        }
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
        let (sender, receiver) = tokio::sync::mpsc::channel(100);
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
        let (sender, receiver) = tokio::sync::mpsc::channel(100);
        
        self.clients[id] = Some(sender);

        BrokerClient::new(id, self.sender.clone(), receiver)
    }
    
    pub async fn run_broker(&mut self) -> Result<(), Box<dyn Error>> {
        loop {
            let Some(message) = self.receiver.recv().await else {
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
                    self.send_response(message, response).await?;
                }
                MessageKind::Broker(BrokerMessage::ConnectToSession) => {
                    self.create_editor_session(message).await?;
                }
                _ => {}
            }
        }
        Ok(())
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
        tokio::spawn(Session::run_session(session_client, message.source));
        self.send_response(message, response).await
    }
}