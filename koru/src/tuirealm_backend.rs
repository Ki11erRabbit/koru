mod events;
mod input;
mod components;

use std::error::Error;
use std::sync::mpsc::{Receiver, Sender};
use std::time::Duration;
use tuirealm::{Application, Attribute, EventListenerCfg, PollStrategy, Sub, SubClause, SubEventClause, Update};
use tuirealm::ratatui::style::Styled;
use tuirealm::terminal::{CrosstermTerminalAdapter, TerminalBridge};
use koru_core::kernel::broker::{BrokerClient, BrokerMessage, GeneralMessage, Message, MessageKind};
use koru_core::kernel::client::{ClientConnectingMessage, ClientConnectingResponse};
use koru_core::kernel::input::KeyPress;
use koru_core::styled_text::{StyledFile};
use crate::tuirealm_backend::events::BrokerPort;

#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub enum UiMessage {
    RegisterBrokerClient(BrokerClient),
    BrokerMessage(Message),
    KeyPress(KeyPress),
    Redraw,
}

#[derive(Debug, Clone, Eq, PartialEq, Hash)]
enum Id {
    Input,
    Buffer,
}

struct App {
    quit: bool,
    redraw: bool,
    terminal: TerminalBridge<CrosstermTerminalAdapter>,
    pub broker_client: BrokerClient,
    session_address: Option<usize>,
    text: StyledFile,
}

impl App {
    pub fn view(&mut self, app: &mut Application<Id, UiMessage, UiMessage>) {
        
        app.attr(&Id::Buffer, Attribute::Text, components::TextView::lines(&self.text)).expect("Invalid attribute");
        
        self.terminal.draw(|frame| {
            app.view(&Id::Buffer, frame, frame.area())
        }).unwrap();
    }
    
    pub fn handle_broker_message(&mut self, msg: Message) -> Result<(), Box<dyn Error>> {
        match msg.kind {
            MessageKind::Broker(BrokerMessage::ConnectToSession) | 
            MessageKind::Broker(BrokerMessage::CreateClient) |
            MessageKind::Broker(BrokerMessage::CreateClientResponse(..)) => {
                Ok(())
            }
            MessageKind::Broker(BrokerMessage::ConnectedToSession(session_id)) => {
                self.session_address = Some(session_id);
                Ok(())
            }
            MessageKind::General(GeneralMessage::Draw(file)) => {
                self.redraw = true;
                self.text = file;
                Ok(())
            }
            _ => Ok(())
        }
    }
}

impl Update<UiMessage> for App {
    fn update(&mut self, msg: Option<UiMessage>) -> Option<UiMessage> {
        match msg {
            Some(UiMessage::BrokerMessage(msg)) => {
                self.handle_broker_message(msg).ok()?;
                None
            }
            Some(UiMessage::RegisterBrokerClient(..)) => None,
            Some(UiMessage::KeyPress(key_press)) => {
                let mut client = self.broker_client.clone();
                let session_address = self.session_address.unwrap();
                koru_core::spawn_task(async move {
                    match client.send(
                        MessageKind::General(GeneralMessage::KeyEvent(key_press)),
                        session_address).await {
                        Ok(..) => {}
                        Err(e) => println!("Error sending key: {}", e),
                    }
                });
                None
            }
            Some(UiMessage::Redraw) => {
                self.redraw = true;
                None
            }
            None => None,
        }
    }
}

fn init_app(broker_client: &mut BrokerClient) -> Application<Id, UiMessage, UiMessage> {
    let mut app = Application::init(
        EventListenerCfg::default()
            .crossterm_input_listener(Duration::from_millis(10), 3)
            .add_port(Box::new(BrokerPort::new(broker_client)), Duration::from_millis(100), 1)
    );
    app
}

pub async fn real_main(
    client_connector: Sender<ClientConnectingMessage>,
    client_receiver: Receiver<ClientConnectingResponse>,
) -> Result<(), Box<dyn Error>> {
    client_connector.send(ClientConnectingMessage::RequestLocalConnection).unwrap();
    let client = client_receiver.recv().unwrap();
    let mut client = match client {
        ClientConnectingResponse::Connection {
            client
        } => {
            client
        }
        _ => {
            return Err(Box::from(String::from("Unable to connect to Koru")));
        }
    };

    client.send(MessageKind::Broker(BrokerMessage::ConnectToSession), 0).await?;
    let mut application = init_app(&mut client);

    application.mount(
        Id::Input, 
        Box::from(input::Input),
        vec![
            Sub::new(SubEventClause::User(UiMessage::BrokerMessage(Message::new(0, 0, MessageKind::Broker(BrokerMessage::Shutdown)))), SubClause::Always),
            Sub::new(SubEventClause::Any, SubClause::Always),
        ]
    ).unwrap();
    
    application.mount(
        Id::Buffer,
        Box::from(components::TextView::new()),
        vec![
            Sub::new(SubEventClause::Any, SubClause::Always),
        ]
    ).expect("Failed to mount textview");
    
    let mut app = App {
        quit: false,
        redraw: true,
        terminal: TerminalBridge::new_crossterm().unwrap(),
        broker_client: client,
        session_address: None,
        text: StyledFile::new()
    };

    let _ = app.terminal.enter_alternate_screen()?;
    let _ = app.terminal.enable_raw_mode()?;
    

    while !app.quit {
        match application.tick(PollStrategy::Once) {
            Err(err) => {
                app.terminal.disable_raw_mode()?;
                app.terminal.leave_alternate_screen()?;
                eprintln!("{}", err);
            }
            
            Ok(messages) => {
                if messages.len() != 0 {
                    for message in messages {
                        _ = app.update(Some(message));
                    }
                }
                
            }
        }
        
        
        if app.redraw {
            let _ = app.view(&mut application);
            app.redraw = false;
        }
    }

    app.terminal.disable_raw_mode()?;
    app.terminal.leave_alternate_screen()?;
    Ok(())
}