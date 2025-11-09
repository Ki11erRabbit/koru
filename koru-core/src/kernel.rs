mod utils;

mod state;
pub mod input;
mod session;
pub mod client;
pub mod broker;
pub(crate) mod buffer;
pub mod scheme_api;

use std::error::Error;
use std::sync::mpsc::{Receiver, Sender};
use futures::future::BoxFuture;
use crate::kernel::broker::Broker;
use crate::kernel::client::{ClientConnectingMessage, ClientConnectingResponse, ClientConnector};
use crate::kernel::scheme_api::SCHEME_RUNTIME;

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
where F: AsyncFnOnce(Sender<ClientConnectingMessage>, Receiver<ClientConnectingResponse>) -> Result<(), Box<dyn Error>>
{
    /*match utils::locate_config_path() {
        Some(config_path) => {
            state::set_config(config_path)
        }
        None => {
            return Err(Box::from(String::from("TODO: implement a first time wizard to set up the editor")));
        }
    }*/
    start_async_runtime(func)
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

    // This is needed to initialize the LazyLock to prevent deadlock
    let _ = SCHEME_RUNTIME.blocking_lock();
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
    
    scheme_api::load_user_config().await;
}


fn start_async_runtime<F>(
    func: F,
) -> Result<(), Box<dyn Error>>
where F: AsyncFnOnce(Sender<ClientConnectingMessage>, Receiver<ClientConnectingResponse>) -> Result<(), Box<dyn Error>>
{
    let tokio_runtime = tokio::runtime::Runtime::new()?;
    let (send_message, recv_message) = std::sync::mpsc::channel();
    let (send_response, recv_response) = std::sync::mpsc::channel();

    let channel_pair = ChannelPair::new(send_response, recv_message);
    // This is needed to initialize the LazyLock to prevent deadlock
    let _ = SCHEME_RUNTIME.blocking_lock();

    let runtime = async move {
        start_runtime(channel_pair).await;
        match func(send_message, recv_response).await {
            Ok(_) => {}
            Err(e) => {
                eprintln!("{}", e);
            }
        }
    };

    tokio_runtime.block_on(runtime);
    Ok(())
}