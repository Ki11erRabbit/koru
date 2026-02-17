mod events;
mod input;
mod components;
mod buffer_state;
pub mod colors;

use std::error::Error;
use std::sync::mpsc::{Receiver, Sender};
use std::time::Duration;
use tabled::Table;
use tuirealm::{Application, AttrValue, Attribute, EventListenerCfg, PollStrategy, Sub, SubClause, SubEventClause, Update};
use tuirealm::ratatui::style::Styled;
use tuirealm::terminal::{CrosstermTerminalAdapter, TerminalBridge};
use koru_core::kernel::broker::{BrokerClient, BrokerMessage, GeneralMessage, Message, MessageKind};
use koru_core::kernel::client::{ClientConnectingMessage, ClientConnectingResponse};
use koru_core::kernel::input::{KeyBuffer, KeyPress};
use crate::tuirealm_backend::components::TextView;
use crate::tuirealm_backend::events::BrokerPort;
use buffer_state::BufferState;
use koru_core::KoruLogger;
use crate::crash_logs::CrashLog;
use crate::tuirealm_backend::colors::ColorDefinitions;

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
    MessageBar,
}

struct App {
    quit: bool,
    crashed: bool,
    redraw: bool,
    terminal: TerminalBridge<CrosstermTerminalAdapter>,
    pub broker_client: BrokerClient,
    session_address: Option<usize>,
    message_bar: String,
    key_buffer: KeyBuffer,
    buffer_state: BufferState,
}

impl App {
    pub fn view(&mut self, app: &mut Application<Id, UiMessage, UiMessage>) {

        app.attr(&Id::Buffer, Attribute::Text, TextView::lines(&self.buffer_state.text)).expect("Invalid attribute");
        app.attr(&Id::Buffer, Attribute::Custom("LineOffset"), AttrValue::Number(self.buffer_state.line_offset as isize)).expect("Invalid attribute");
        app.attr(&Id::Buffer, Attribute::Custom("ColumnOffset"), AttrValue::Number(self.buffer_state.column_offset as isize)).expect("Invalid attribute");
        app.attr(&Id::MessageBar, Attribute::Text, AttrValue::String(self.message_bar.clone())).expect("Invalid attribute");
        
        self.terminal.draw(|frame| {
            
            let mut text_area = frame.area();
            text_area.height -= 1;
            self.buffer_state.line_count = text_area.height as usize;
            self.buffer_state.column_count = text_area.width as usize;
            
            let mut message_area = frame.area();
            message_area.height = 1;
            message_area.y += text_area.height;
            
            app.view(&Id::Buffer, frame, text_area);
            app.view(&Id::MessageBar, frame, message_area);
            
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
                let mut client = self.broker_client.clone();
                let session_address = self.session_address.unwrap();
                koru_core::spawn_task(async move {
                    match client.send_async(
                        MessageKind::General(GeneralMessage::RequestMainCursor),
                        session_address).await {
                        Ok(..) => {}
                        Err(e) => println!("Error sending request main cursor: {}", e),
                    }
                });
                Ok(())
            }
            MessageKind::General(GeneralMessage::Draw(file)) => {
                self.redraw = true;
                self.buffer_state.text = file;
                Ok(())
            }
            MessageKind::General(GeneralMessage::UpdateMessageBar(bar)) => {
                self.redraw = true;
                self.message_bar = bar;
                Ok(())
            }
            MessageKind::General(GeneralMessage::FlushKeyBuffer) => {
                self.key_buffer.clear();
                Ok(())
            }
            MessageKind::General(GeneralMessage::MainCursorPosition(line, col)) => {
                self.buffer_state.line = line;
                self.buffer_state.col = col;
                self.buffer_state.scroll_view();
                self.redraw = true;
                Ok(())
            }
            MessageKind::General(GeneralMessage::HideCommandBar) => {
                Ok(())
            }
            MessageKind::General(GeneralMessage::ShowCommandBar) => {
                Ok(())
            }
            MessageKind::General(GeneralMessage::UpdateCommandBar {
                prefix,
                body,
                suffix,
                                 }) => {
                self.message_bar = prefix + body.as_str() + suffix.as_str();
                Ok(())
            }
            MessageKind::Broker(BrokerMessage::Crash) => {
                self.crashed = true;
                self.quit = true;
                Ok(())
            }
            MessageKind::General(GeneralMessage::SetColorDef(definition)) => {
                let (color_type, color_value) = definition.to_tuple();
                ColorDefinitions::insert(color_type, color_value);
                Ok(())
            }
            MessageKind::General(GeneralMessage::Quit) => {
                self.quit = true;
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
                    match client.send_async(
                        MessageKind::General(GeneralMessage::KeyEvent(key_press)),
                        session_address).await {
                        Ok(..) => {}
                        Err(e) => println!("Error sending key: {}", e),
                    }
                    match client.send_async(
                        MessageKind::General(GeneralMessage::RequestMainCursor),
                        session_address).await {
                        Ok(..) => {}
                        Err(e) => println!("Error sending request main cursor: {}", e),
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
            .add_port(Box::new(BrokerPort::new(broker_client)), Duration::from_millis(5), 1)
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

    client.send_async(MessageKind::Broker(BrokerMessage::ConnectToSession), 0).await?;
    let mut application = init_app(&mut client);

    application.mount(
        Id::Input, 
        Box::from(input::Input),
        vec![
            Sub::new(SubEventClause::User(UiMessage::BrokerMessage(Message::new(0, 0, MessageKind::Broker(BrokerMessage::Shutdown)))), SubClause::Always),
            Sub::new(SubEventClause::Any, SubClause::Always),
        ]
    )?;
    
    application.mount(
        Id::Buffer,
        Box::from(components::TextView::new()),
        vec![]
    ).expect("Failed to mount textview");

    application.mount(
        Id::MessageBar,
        Box::from(components::MessageBar::new()),
        vec![
            Sub::new(SubEventClause::Any, SubClause::Always),
        ]
    ).expect("Failed to mount messsagebar");
    
    let mut app = App {
        crashed: false,
        quit: false,
        redraw: true,
        terminal: TerminalBridge::new_crossterm()?,
        broker_client: client,
        session_address: None,
        message_bar: String::new(),
        key_buffer: KeyBuffer::new(),
        buffer_state: BufferState::default(),
    };

    let _ = app.terminal.enter_alternate_screen()?;
    let _ = app.terminal.enable_raw_mode()?;
    
    while !app.quit {
        match application.tick(PollStrategy::TryFor(Duration::from_millis(16))) {
            Err(err) => {
                app.quit = true;
                app.crashed = true;
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

    if app.crashed {
        let logs = KoruLogger::all_logs_async().await;
        let logs = logs.into_iter().map(|log| {
            let (level, timestamp, module_path, file, message) = log.format("%H:%M:%S").expect("invalid format string");
            CrashLog::new(level, timestamp, module_path, file, message)
        }).collect::<Vec<_>>();
        let logs: Vec<CrashLog> = logs.iter().rev().take(100).cloned().collect();
        let table = Table::new(logs);
        let table = table.to_string();
        println!("Koru has crashed. Here are the last 100 logs\n");
        println!("{}", table);
    }

    Ok(())
}