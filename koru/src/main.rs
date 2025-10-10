use std::error::Error;
use std::pin::pin;
use std::sync::mpsc::{Receiver, Sender};
use futures::future::BoxFuture;
use iced::application::View;
use iced::{Element, Task};
use iced::futures::FutureExt;
use iced::widget::text;
use koru_core::kernel::broker::BrokerClient;
use koru_core::kernel::client::{ClientConnectingMessage, ClientConnectingResponse};
use koru_core::koru_main_ui;

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
    RegisterBrokerClient(BrokerClient),
    ConnectToKernel,
}

#[derive(Default)]
struct App {
    kernel_runtime: Option<BoxFuture<'static, ()>>,
    client_connection: Option<(Sender<ClientConnectingMessage>, Receiver<ClientConnectingResponse>)>,
    broker_client: Option<BrokerClient>,
}

impl App {
    fn new(
        kernel_runtime: BoxFuture<'static, ()>,
        client_connector: Sender<ClientConnectingMessage>,
        client_receiver: Receiver<ClientConnectingResponse>,
    ) -> Self {
        App {
            kernel_runtime: Some(kernel_runtime),
            client_connection: Some((client_connector, client_receiver)),
            broker_client: None,
        }
    }
    
    fn update(&mut self, message: UiMessage) -> Task<UiMessage>{
        
        match message {
            UiMessage::RunKernelRuntime => {
                if let Some(kernel_runtime) = self.kernel_runtime.take() {
                    Task::future(async move {
                        kernel_runtime.await;

                        UiMessage::ConnectToKernel
                    })
                } else {
                    unreachable!()
                }
            }
            UiMessage::RegisterBrokerClient(client) => {
                self.broker_client = Some(client);
                Task::none()
            }
            UiMessage::ConnectToKernel => {
                if let Some(client_connector) = self.client_connection.take() {
                    let client_connector = ClientConnector::new(client_connector);
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
                } else {
                    unreachable!()
                }
            }
        }
    }
    
    fn view(&self) -> Element<UiMessage> {
        if self.broker_client.is_some() {
            text("Connected to Koru").size(20).into()
        } else {
            text("Koru").size(20).into()
        }
    }
}

fn true_main(
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


fn main() -> Result<(), Box<dyn Error>> {
    koru_main_ui(true_main)
}