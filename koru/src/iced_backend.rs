use std::error::Error;
use std::hint::unreachable_unchecked;
use std::sync::mpsc::{Receiver, Sender};
use futures::future::BoxFuture;
use futures::SinkExt;
use iced::application::View;
use iced::{Element, Task};
use iced::widget::text;
use koru_core::kernel::broker::{BrokerClient, BrokerMessage, Message, MessageKind};
use koru_core::kernel::client::{ClientConnectingMessage, ClientConnectingResponse};

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

#[derive(Debug)]
enum UiMessage {
    RunKernelRuntime,
    ConnectToKernel,
    RegisterBrokerClient(BrokerClient),
    ConnectToSession,
    BrokerMessage(Message),
}

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
        }
    }

    fn update(&mut self, message: UiMessage) -> Task<UiMessage>{

        match message {
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
                        std::mem::swap(client, &mut stream_client);
                        // Cloning a client only gives us the sender.
                        // Therefore, we must switch the two around
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
            _ => Task::none()
        }
    }

    fn view(&self) -> Element<UiMessage> {
        match &self.initialization_state {
            AppInitializationState::Initialized(_) => {
                text("Connected to Koru").size(20).into()
            }
            _ => {
                text("Koru").size(20).into()
            }
        }
    }
}

pub fn true_main(
    client_connector: Sender<ClientConnectingMessage>,
    client_receiver: Receiver<ClientConnectingResponse>,
    runtime_future: BoxFuture<'static, ()>
) -> Result<(), Box<dyn Error>> {
    iced::application("Koru", App::update, App::view).run_with(move || {
        (App::new(runtime_future, client_connector, client_receiver), Task::future(async {
            UiMessage::RunKernelRuntime
        }))
    })?;
    Ok(())
}
