use crate::kernel::broker::{BrokerClient, BrokerMessage, MessageKind};

pub enum ClientConnectingMessage {
    RequestLocalConnection,
}

pub enum ClientConnectingResponse {
    Connection {
        client: BrokerClient,
    }
}



pub struct ClientConnector {
    client: BrokerClient
}

impl ClientConnector {
    pub fn new(client: BrokerClient) -> ClientConnector {
        Self { client }
    }
    
    pub async fn run_connector(
        &mut self, 
        local_client: Option<(std::sync::mpsc::Sender<ClientConnectingResponse>, std::sync::mpsc::Receiver<ClientConnectingMessage>)>
    ) -> Result<(), Box<dyn std::error::Error>> {
        if let Some((sender, receiver)) = local_client {
            match receiver.recv()? {
                ClientConnectingMessage::RequestLocalConnection => {
                    self.client.send(MessageKind::Broker(BrokerMessage::CreateClient), 0).await?;
                    match self.client.recv().await {
                        Some(msg) => {
                            match msg.kind {
                                MessageKind::Broker(BrokerMessage::CreateClientResponse(client)) => {
                                    let response = ClientConnectingResponse::Connection {
                                        client,
                                    };
                                    sender.send(response)?;
                                }
                                _ => unreachable!("Unexpected message kind"),
                            }
                        }
                        None => {
                            panic!("Broker Thread Died");
                        }
                    }
                }
            }
        }
        loop {}
    }
}