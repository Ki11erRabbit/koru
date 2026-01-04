mod styled_text;
mod buffer_state;

use std::error::Error;
use std::sync::mpsc::{Receiver, Sender};
use futures::future::BoxFuture;
use futures::SinkExt;
use iced::{Element, Task};
use iced::keyboard::Key;
use iced::keyboard::key::Named;
use iced::widget::{column, scrollable, text};
use iced_core::keyboard::Modifiers;
use iced_core::{Alignment, Length};
use iced_core::text::{Fragment, Span, Wrapping};
use iced_futures::Subscription;
use koru_core::kernel::broker::{BrokerClient, BrokerMessage, GeneralMessage, Message, MessageKind};
use koru_core::kernel::client::{ClientConnectingMessage, ClientConnectingResponse};
use koru_core::kernel::input::{ControlKey, KeyBuffer, KeyPress, KeyValue, ModifierKey};
use buffer_state::BufferState;

use iced_core::window::Id as WindowId;
use tabled::Table;
use koru_core::{KoruLogger, LogEntry};
use crate::crash_logs::CrashLog;

#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub enum UiMessage {
    Nop,
    RunKernelRuntime,
    ConnectToKernel,
    RegisterBrokerClient(BrokerClient),
    ConnectToSession,
    BrokerMessage(Message),
    KeyPress(KeyPress),
    CloseEvent(WindowId),
    CloseRequest(WindowId),
    CrashLog(Vec<CrashLog>)
}


struct ClientConnector {
    sender: Sender<ClientConnectingMessage>,
    receiver: Receiver<ClientConnectingResponse>,
}

impl ClientConnector {
    fn new((sender, receiver): (Sender<ClientConnectingMessage>, Receiver<ClientConnectingResponse>)) -> Self {
        Self { sender, receiver }
    }

    fn to_tuple(self) -> (Sender<ClientConnectingMessage>, Receiver<ClientConnectingResponse>) {
        (self.sender, self.receiver)
    }
}

unsafe impl Send for ClientConnector {}
unsafe impl Sync for ClientConnector {}

enum AppInitializationState {
    KernelNotStarted {
        kernel_runtime: BoxFuture<'static, ()>,
        client_connection: (Sender<ClientConnectingMessage>, Receiver<ClientConnectingResponse>),
    },
    ClientNotConnected {
        client_connection: (Sender<ClientConnectingMessage>, Receiver<ClientConnectingResponse>),
    },
    ConnectingToSession(BrokerClient),
    Initialized(BrokerClient),
    Blank,
    Crashed(Option<Vec<CrashLog>>),
}


struct App {
    initialization_state: AppInitializationState,
    session_address: Option<usize>,
    message_bar: String,
    key_buffer: KeyBuffer,
    buffer_state: BufferState,
}

impl App {
    fn new(
        kernel_runtime: BoxFuture<'static, ()>,
        client_connector: Sender<ClientConnectingMessage>,
        client_receiver: Receiver<ClientConnectingResponse>,
    ) -> Self {
        App {
            initialization_state: AppInitializationState::KernelNotStarted {
                kernel_runtime,
                client_connection: (client_connector, client_receiver),
            },
            session_address: None,
            message_bar: String::new(),
            key_buffer: KeyBuffer::new(),
            buffer_state: BufferState::default(),
        }
    }

    fn send_client_messages(&mut self, messages: Vec<MessageKind>) -> Task<UiMessage> {
        match &self.initialization_state {
            AppInitializationState::Initialized(client) => {
                let destination = self.session_address.unwrap();
                let mut client = client.clone();
                Task::future(async move {
                    for message in messages {
                        match client.send_async(message, destination).await {
                            _ => {}
                        }
                    }

                    UiMessage::Nop
                })
            }
            _ => unreachable!("We shouldn't in any other state at this point."),
        }
    }

    fn setup_client_stream(&mut self) -> Task<UiMessage> {
        let state = std::mem::replace(&mut self.initialization_state, AppInitializationState::Blank);
        match state {
            AppInitializationState::ClientNotConnected { client_connection } => {
                let client_connector = ClientConnector::new(client_connection);
                Task::future(async move {
                    let (sender, receiver) = client_connector.to_tuple();
                    sender.send(ClientConnectingMessage::RequestLocalConnection).unwrap();
                    let response = receiver.recv().unwrap();
                    match response {
                        ClientConnectingResponse::Connection { client } => {
                            UiMessage::RegisterBrokerClient(client)
                        }
                        _ => unreachable!(),
                    }
                })
            }
            _ => unreachable!("We shouldn't in any other state at this point."),
        }
    }

    fn start_kernel(&mut self) -> Task<UiMessage> {
        let state = std::mem::replace(&mut self.initialization_state, AppInitializationState::Blank);
        match state {
            AppInitializationState::KernelNotStarted { kernel_runtime, client_connection } => {
                self.initialization_state = AppInitializationState::ClientNotConnected { client_connection };
                Task::future(async move {
                    kernel_runtime.await;

                    UiMessage::ConnectToKernel
                })
            }
            _ => unreachable!("We shouldn't in any other state at this point."),
        }
    }

    fn update(&mut self, message: UiMessage) -> Task<UiMessage> {
        match message {
            UiMessage::Nop => Task::none(),
            UiMessage::RunKernelRuntime => {
                self.start_kernel()
            }
            UiMessage::ConnectToKernel => {
                self.setup_client_stream()
            }
            UiMessage::RegisterBrokerClient(client) => {
                self.initialization_state = AppInitializationState::ConnectingToSession(client);
                Task::future(async {
                    UiMessage::ConnectToSession
                })
            }
            UiMessage::ConnectToSession => {
                match &mut self.initialization_state {
                    AppInitializationState::ConnectingToSession(client) => {
                        let mut stream_client = client.clone();
                        // Cloning a client only gives us the sender.
                        // Therefore, we must switch the two around
                        std::mem::swap(client, &mut stream_client);
                        Task::stream(iced::stream::channel(100, async move |mut output| {
                            stream_client.send_async(MessageKind::Broker(BrokerMessage::ConnectToSession), 0).await.unwrap();
                            loop {
                                match stream_client.recv_async().await {
                                    Some(msg) => {
                                        output.send(UiMessage::BrokerMessage(msg)).await.unwrap();
                                    }
                                    _ => {}
                                }
                            }
                        }))
                    }
                    _ => unreachable!("We shouldn't in any other state at this point."),
                }
            }
            UiMessage::BrokerMessage(msg) => {
                self.handle_broker_message(msg)
            }
            UiMessage::KeyPress(key_press) => {
                self.send_client_messages(vec![
                    MessageKind::General(GeneralMessage::KeyEvent(key_press)),
                    MessageKind::General(GeneralMessage::RequestMainCursor)
                ])
            }
            UiMessage::CloseEvent(window_id) => {
                self.send_client_messages(vec![
                    MessageKind::Broker(BrokerMessage::Shutdown)
                ]).chain(iced::window::close(window_id))
            }
            UiMessage::CloseRequest(window_id) => {
                self.send_client_messages(vec![
                    MessageKind::Broker(BrokerMessage::Shutdown)
                ]).chain(iced::window::close(window_id))
            }
            UiMessage::CrashLog(logs) => {
                match &mut self.initialization_state {
                    AppInitializationState::Crashed(_) => {
                        self.initialization_state = AppInitializationState::Crashed(Some(logs));
                    }
                    _ => unreachable!("We shouldn't in any other state at this point."),
                }
                Task::done(UiMessage::Nop)
            }
        }
    }

    fn handle_broker_message(&mut self, message: Message) -> Task<UiMessage> {
        match message.kind {
            MessageKind::Broker(BrokerMessage::ConnectedToSession(session_address)) => {
                self.session_address = Some(session_address);
                let state = std::mem::replace(&mut self.initialization_state, AppInitializationState::Blank);
                match state {
                    AppInitializationState::ConnectingToSession(client) => {
                        self.initialization_state = AppInitializationState::Initialized(client)
                    }
                    _ => unreachable!("We shouldn't in any other state at this point.")
                }
                Task::none()
            }
            MessageKind::General(GeneralMessage::Draw(styled_file)) => {
                self.buffer_state.text = styled_file;
                self.send_client_messages(vec![
                    MessageKind::General(GeneralMessage::RequestMainCursor)
                ])
            }
            MessageKind::General(GeneralMessage::UpdateMessageBar(message_bar)) => {
                self.message_bar = message_bar;
                Task::none()
            }
            MessageKind::General(GeneralMessage::FlushKeyBuffer) => {
                self.key_buffer.clear();
                Task::none()
            }
            MessageKind::General(GeneralMessage::MainCursorPosition(line, col)) => {
                self.buffer_state.col = col;
                self.buffer_state.line = line;
                self.buffer_state.scroll_view();
                Task::none()
            }
            MessageKind::General(GeneralMessage::ShowCommandBar) => {
                Task::none()
            }
            MessageKind::General(GeneralMessage::HideCommandBar) => {
                Task::none()
            }
            MessageKind::General(GeneralMessage::UpdateCommandBar(commandbar)) => {
                self.message_bar = commandbar;
                Task::none()
            }
            MessageKind::Broker(BrokerMessage::Crash) => {
                match &mut self.initialization_state {
                    AppInitializationState::Initialized(_) => {
                        self.initialization_state = AppInitializationState::Crashed(None);
                        Task::future(async {
                            let logs = KoruLogger::all_logs_async().await;
                            let logs = logs.into_iter().map(|log| {
                                let (level, timestamp, module_path, file, message) = log.format("%H:%M:%S").expect("invalid format string");
                                CrashLog::new(level, timestamp, module_path, file, message)
                            }).collect::<Vec<_>>();
                            UiMessage::CrashLog(logs)
                        })
                    }
                    _ => unreachable!("We shouldn't in any other state at this point."),
                }
            }
            _ => Task::none()
        }
    }

    fn view(&self) -> Element<UiMessage> {
        match &self.initialization_state {
            AppInitializationState::Initialized(_) => {
                column!(
                    styled_text::rich(&self.buffer_state.text.lines(), self.buffer_state.line_offset, self.buffer_state.column_offset, self.buffer_state.text_metrics_callback())
                        .font(iced::font::Font::MONOSPACE)
                        .height(Length::Fill),
                    text(&self.message_bar)
                ).into()
            }
            AppInitializationState::Crashed(None) => {
                text("Waiting for crash log").size(20).into()
            }
            AppInitializationState::Crashed(Some(logs)) => {
                let logs: Vec<CrashLog> = logs.iter().rev().take(100).cloned().collect();
                let table = Table::new(logs);
                let table = table.to_string();

                let lines = table.lines().map(|text| text.to_string() + "\n").collect();
                let title: Element<_> = text("Koru has crashed. Here are the last 100 logs").size(20).align_x(Alignment::Center).into();
                let body: Element<_> = styled_text::rich_simple(lines)
                    .font(iced::font::Font::MONOSPACE)
                    .wrapping(Wrapping::None)
                    .into();
                let body = scrollable(body).direction(scrollable::Direction::Both {
                    vertical: Default::default(),
                    horizontal: Default::default(),
                })
                    .height(Length::Fill)
                    .width(Length::Fill);
                column!(
                    title,
                    body
                ).into()
            }
            _ => {
                text("Koru").size(20).into()
            }
        }
    }

    fn on_key_press_handler(key: Key, mods: Modifiers) -> Option<UiMessage> {
        let mut modifiers = ModifierKey::empty();
        if mods.logo() {
            modifiers |= ModifierKey::Meta;
        }
        if mods.shift() {
            modifiers |= ModifierKey::Shift;
        }
        if mods.control() {
            modifiers |= ModifierKey::Control;
        }
        if mods.alt() {
            modifiers |= ModifierKey::Alt;
        }
        let key = match key {
            Key::Character(c) => {
                KeyValue::CharacterKey(c.to_string().into_boxed_str())
            }
            Key::Named(Named::Enter) => KeyValue::ControlKey(ControlKey::Enter),
            Key::Named(Named::Tab) => KeyValue::ControlKey(ControlKey::Tab),
            Key::Named(Named::Space) => KeyValue::ControlKey(ControlKey::Space),
            Key::Named(Named::Escape) => KeyValue::ControlKey(ControlKey::Escape),
            Key::Named(Named::Backspace) => KeyValue::ControlKey(ControlKey::Backspace),
            Key::Named(Named::Delete) => KeyValue::ControlKey(ControlKey::Delete),
            Key::Named(Named::ArrowRight) => KeyValue::ControlKey(ControlKey::Right),
            Key::Named(Named::ArrowLeft) => KeyValue::ControlKey(ControlKey::Left),
            Key::Named(Named::ArrowDown) => KeyValue::ControlKey(ControlKey::Down),
            Key::Named(Named::ArrowUp) => KeyValue::ControlKey(ControlKey::Up),
            Key::Named(Named::PageUp) => KeyValue::ControlKey(ControlKey::PageUp),
            Key::Named(Named::PageDown) => KeyValue::ControlKey(ControlKey::PageDown),
            Key::Named(Named::Home) => KeyValue::ControlKey(ControlKey::Home),
            Key::Named(Named::End) => KeyValue::ControlKey(ControlKey::End),
            Key::Named(Named::F1) => KeyValue::ControlKey(ControlKey::F1),
            Key::Named(Named::F2) => KeyValue::ControlKey(ControlKey::F2),
            Key::Named(Named::F3) => KeyValue::ControlKey(ControlKey::F3),
            Key::Named(Named::F4) => KeyValue::ControlKey(ControlKey::F4),
            Key::Named(Named::F5) => KeyValue::ControlKey(ControlKey::F5),
            Key::Named(Named::F6) => KeyValue::ControlKey(ControlKey::F6),
            Key::Named(Named::F7) => KeyValue::ControlKey(ControlKey::F7),
            Key::Named(Named::F8) => KeyValue::ControlKey(ControlKey::F8),
            Key::Named(Named::F9) => KeyValue::ControlKey(ControlKey::F9),
            Key::Named(Named::F10) => KeyValue::ControlKey(ControlKey::F10),
            Key::Named(Named::F11) => KeyValue::ControlKey(ControlKey::F11),
            Key::Named(Named::F12) => KeyValue::ControlKey(ControlKey::F12),
            Key::Named(Named::F13) => KeyValue::ControlKey(ControlKey::F13),
            Key::Named(Named::F14) => KeyValue::ControlKey(ControlKey::F14),
            Key::Named(Named::F15) => KeyValue::ControlKey(ControlKey::F15),
            Key::Named(Named::F16) => KeyValue::ControlKey(ControlKey::F16),
            Key::Named(Named::F17) => KeyValue::ControlKey(ControlKey::F17),
            Key::Named(Named::F18) => KeyValue::ControlKey(ControlKey::F18),
            Key::Named(Named::F19) => KeyValue::ControlKey(ControlKey::F19),
            Key::Named(Named::F20) => KeyValue::ControlKey(ControlKey::F20),
            Key::Named(Named::F21) => KeyValue::ControlKey(ControlKey::F21),
            Key::Named(Named::F22) => KeyValue::ControlKey(ControlKey::F22),
            Key::Named(Named::F23) => KeyValue::ControlKey(ControlKey::F23),
            Key::Named(Named::F24) => KeyValue::ControlKey(ControlKey::F24),
            Key::Named(Named::F25) => KeyValue::ControlKey(ControlKey::F25),
            Key::Named(Named::F26) => KeyValue::ControlKey(ControlKey::F26),
            Key::Named(Named::F27) => KeyValue::ControlKey(ControlKey::F27),
            Key::Named(Named::F28) => KeyValue::ControlKey(ControlKey::F28),
            Key::Named(Named::F29) => KeyValue::ControlKey(ControlKey::F29),
            Key::Named(Named::F30) => KeyValue::ControlKey(ControlKey::F30),
            Key::Named(Named::F31) => KeyValue::ControlKey(ControlKey::F31),
            Key::Named(Named::F32) => KeyValue::ControlKey(ControlKey::F32),
            Key::Named(Named::F33) => KeyValue::ControlKey(ControlKey::F33),
            Key::Named(Named::F34) => KeyValue::ControlKey(ControlKey::F34),
            Key::Named(Named::F35) => KeyValue::ControlKey(ControlKey::F35),
            _ => return None,
        };

        Some(UiMessage::KeyPress(KeyPress::new(key, modifiers)))
    }

    fn subscription(&self) -> Subscription<UiMessage> {
        Subscription::batch([
            iced::keyboard::on_key_press(Self::on_key_press_handler),
            iced::window::close_events().map(UiMessage::CloseEvent),
            iced::window::close_requests().map(UiMessage::CloseRequest),
        ])
    }
}

pub fn true_main(
    client_connector: Sender<ClientConnectingMessage>,
    client_receiver: Receiver<ClientConnectingResponse>,
    runtime_future: BoxFuture<'static, ()>
) -> Result<(), Box<dyn Error>> {
    iced::application("Koru", App::update, App::view).subscription(App::subscription).run_with(move || {
        (App::new(runtime_future, client_connector, client_receiver), Task::future(async {
            UiMessage::RunKernelRuntime
        }))
    })?;
    Ok(())
}
