mod styled_text;

use std::error::Error;
use std::sync::mpsc::{Receiver, Sender};
use futures::future::BoxFuture;
use futures::SinkExt;
use iced::{Element, Task};
use iced::keyboard::Key;
use iced::keyboard::key::Named;
use iced::widget::{column, text, Scrollable};
use iced_core::Length;
use iced_futures::Subscription;
use koru_core::kernel::broker::{BrokerClient, BrokerMessage, GeneralMessage, Message, MessageKind};
use koru_core::kernel::client::{ClientConnectingMessage, ClientConnectingResponse};
use koru_core::kernel::input::{ControlKey, KeyPress, KeyValue, ModifierKey};
use koru_core::styled_text::{StyledFile};

#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub enum UiMessage {
    Nop,
    RunKernelRuntime,
    ConnectToKernel,
    RegisterBrokerClient(BrokerClient),
    ConnectToSession,
    BrokerMessage(Message),
    KeyPress(KeyPress),
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
}


struct App {
    initialization_state: AppInitializationState,
    session_address: Option<usize>,
    text: StyledFile,
    message_bar: String,
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
            text: StyledFile::new(),
            message_bar: String::from("Hello world!"),
        }
    }

    fn update(&mut self, message: UiMessage) -> Task<UiMessage>{

        match message {
            UiMessage::Nop => Task::none(),
            UiMessage::RunKernelRuntime => {
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
            UiMessage::ConnectToKernel => {
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
                            stream_client.send(MessageKind::Broker(BrokerMessage::ConnectToSession), 0).await.unwrap();
                            loop {
                                match stream_client.recv().await {
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
                match &self.initialization_state {
                    AppInitializationState::Initialized(client) => {
                        let destination = self.session_address.unwrap();
                        let mut client = client.clone();
                        Task::future(async move {
                            match client.send(MessageKind::General(GeneralMessage::KeyEvent(key_press)), destination).await {
                                _ => {}
                            }
                            UiMessage::Nop
                        })
                    }
                    _ => Task::none(),
                }
            }
        }
    }

    fn handle_broker_message(&mut self, message: Message) -> Task<UiMessage> {
        match message.kind {
            MessageKind::Broker(BrokerMessage::ConnectedToSession(session_address)) => {
                println!("connected to session");
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
                self.text = styled_file;
                Task::none()
            }
            MessageKind::General(GeneralMessage::UpdateMessageBar(message_bar)) => {
                println!("update message bar");
                self.message_bar = message_bar;
                Task::none()
            }
            _ => Task::none()
        }
    }

    fn view(&self) -> Element<UiMessage> {
        match &self.initialization_state {
            AppInitializationState::Initialized(_) => {
                column!(
                    Scrollable::new(styled_text::rich(&self.text.lines())
                        .font(iced::font::Font::MONOSPACE))
                        .height(Length::Fill)
                        .width(Length::Fill),
                    text(&self.message_bar)
                ).into()
            }
            _ => {
                text("Koru").size(20).into()
            }
        }
    }

    fn subscription(&self) -> Subscription<UiMessage> {
        iced::keyboard::on_key_press(|key, mods| {
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
                    match c.as_str().chars().next() {
                        Some(c) => {
                            KeyValue::CharacterKey(c)
                        }
                        _ => panic!("invalid character key"),
                    }
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
        })
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
