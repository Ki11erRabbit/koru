use std::sync::mpsc::{Receiver, TryRecvError};
use tuirealm::listener::{ListenerResult, Poll};
use tuirealm::ListenerError;
use koru_core::kernel::broker::BrokerClient;
use crate::tuirealm_backend::{App, UiMessage};


pub struct BrokerPort {
    receiver: Receiver<UiMessage>,
}

impl BrokerPort {
    pub fn new(broker_client: &mut BrokerClient) -> BrokerPort {
        let mut client = broker_client.clone();
        std::mem::swap(broker_client, &mut client);
        let (sender, receiver) = std::sync::mpsc::channel();
        
        koru_core::spawn_task(async move {
            loop {
                match client.recv().await {
                    Some(msg) => {
                        sender.send(UiMessage::BrokerMessage(msg)).unwrap();
                    }
                    None => break,
                }
            }
        });
        
        //let value = receiver.recv().unwrap();
        //println!("BrokerPort Sent: {:?}", value);
        
        BrokerPort { receiver }
    }
}

impl Poll<UiMessage> for BrokerPort {
    fn poll(&mut self) -> ListenerResult<Option<tuirealm::Event<UiMessage>>> {
        //println!("BrokerPort Poll");
        match self.receiver.try_recv() {
            Ok(msg) => {
                Ok(Some(tuirealm::Event::User(msg)))
            }
            Err(TryRecvError::Disconnected) => {
                Err(ListenerError::ListenerDied)
            },
            _ => Ok(None),
        }
    }
}