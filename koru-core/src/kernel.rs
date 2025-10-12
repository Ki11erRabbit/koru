mod utils;
mod state;
pub mod input;
mod lua_api;
mod files;
mod session;
pub mod client;
pub mod broker;

use std::error::Error;
use std::io;
use std::sync::mpsc::{Receiver, Sender};
use futures::future::BoxFuture;
use crate::kernel::broker::Broker;
use crate::kernel::client::{ClientConnectingMessage, ClientConnectingResponse, ClientConnector};

struct ChannelPair {
    sender: Sender<ClientConnectingResponse>,
    receiver: Receiver<ClientConnectingMessage>,
}

impl ChannelPair {
    fn new(sender: Sender<ClientConnectingResponse>, receiver: Receiver<ClientConnectingMessage>) -> Self {
        Self { sender, receiver }
    }
    
    fn to_tuple(self) -> (Sender<ClientConnectingResponse>, Receiver<ClientConnectingMessage>) {
        (self.sender, self.receiver)
    }
}

unsafe impl Send for ChannelPair {}
unsafe impl Sync for ChannelPair {}


/// Starts the Kernel's Runtime
///
/// This will also start an async runtime for the Kernel's Runtime.
/// We then hand control to the caller via `func` to then start the ui runtime.
///
/// This should **NOT** be called if `func` will start an async runtime.
pub fn start_kernel<F>(func: F) -> Result<(), Box<dyn Error>>
where F: FnOnce(Sender<ClientConnectingMessage>, Receiver<ClientConnectingResponse>) -> Result<(), Box<dyn Error>>
{
    /*match utils::locate_config_path() {
        Some(config_path) => {
            state::set_config(config_path)
        }
        None => {
            return Err(Box::from(String::from("TODO: implement a first time wizard to set up the editor")));
        }
    }*/

    let (send_message, recv_message) = std::sync::mpsc::channel();
    let (send_response, recv_response) = std::sync::mpsc::channel();

    start_async_runtime(Some((send_response, recv_message)))?;

    func(send_message, recv_response)
}

/// Starts the Kernel's Runtime
///
/// This will not start an async runtime.
/// We then hand control to the caller via `func` to then start the ui runtime.
/// We also pass in a future that will start the Kernel's runtime.
/// This should be awaited as soon as possible to prevent a deadlock from the kernel's runtime not being ready yet.
///
/// If `func` doesn't start an async runtime, then you **SHOULDN'T** call this function.
pub fn start_kernel_existing_runtime<F>(func: F) -> Result<(), Box<dyn Error>>
where F: FnOnce(Sender<ClientConnectingMessage>, Receiver<ClientConnectingResponse>, BoxFuture<'static, ()>) -> Result<(), Box<dyn Error>>
{
    /*match utils::locate_config_path() {
        Some(config_path) => {
            state::set_config(config_path)
        }
        None => {
            return Err(Box::from(String::from("TODO: implement a first time wizard to set up the editor")));
        }
    }*/

    let (send_message, recv_message) = std::sync::mpsc::channel();
    let (send_response, recv_response) = std::sync::mpsc::channel();
    
    let channel_pair = ChannelPair::new(send_response, recv_message);

    let runtime = start_runtime(channel_pair);

    func(send_message, recv_response, Box::pin(runtime))
}

async fn start_runtime(pair: ChannelPair) {
    println!("Starting Koru Kernel");
    let mut broker = Broker::new();
    let connector_client = broker.create_client();

    tokio::spawn(async move {
        match broker.run_broker().await {
            Ok(_) => (),
            Err(e) => {
                eprintln!("{}", e);
            }
        }
    });

    let mut client_connector = ClientConnector::new(connector_client);
    let channel_pair = pair.to_tuple();

    tokio::spawn(async move {
        match client_connector.run_connector(Some(channel_pair)).await {
            Ok(()) => {}
            Err(e) => {
                eprintln!("{}", e);
            }
        }
    });
}


fn start_async_runtime(
    local_client: Option<(Sender<ClientConnectingResponse>, Receiver<ClientConnectingMessage>)>
) -> io::Result<()> {
    let tokio_runtime = tokio::runtime::Runtime::new()?;

    let mut broker = Broker::new();
    let connector_client = broker.create_client();

    let _ = std::thread::Builder::new()
        .name("runtime".into())
        .spawn(move || {
            _ = tokio_runtime.block_on(async move {
                let mut client_connector = ClientConnector::new(connector_client);

                tokio::spawn(async move {
                    match client_connector.run_connector(local_client).await {
                        Ok(()) => {}
                        Err(e) => {
                            eprintln!("{}", e);
                        }
                    }
                });
                tokio::spawn(async move {
                    match broker.run_broker().await {
                        Ok(_) => (),
                        Err(e) => {
                            eprintln!("{}", e);
                        }
                    }
                });
            });
            loop {}
        })?;



    Ok(())
}