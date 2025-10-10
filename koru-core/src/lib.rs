pub mod kernel;

use std::error::Error;
use std::sync::mpsc::{Receiver, Sender};
use futures::future::BoxFuture;
use crate::kernel::client::{ClientConnectingMessage, ClientConnectingResponse};


/// Starts the Kernel's Runtime
///
/// This will not start an async runtime.
/// We then hand control to the caller via `ui_logic` to then start the ui runtime.
/// We also pass in a future that will start the Kernel's runtime.
/// This should be awaited as soon as possible to prevent a deadlock from the kernel's runtime not being ready yet.
///
/// If `ui_logic` doesn't start an async runtime, then you **SHOULDN'T** call this function.
pub fn koru_main_ui<F>(ui_logic: F) -> Result<(), Box<dyn Error>>
where F: FnOnce(Sender<ClientConnectingMessage>, Receiver<ClientConnectingResponse>, BoxFuture<'static, ()>) -> Result<(), Box<dyn Error>>
{
    kernel::start_kernel_existing_runtime(ui_logic)
}

/// Starts the Kernel's Runtime
///
/// This will also start an async runtime for the Kernel's Runtime.
/// We then hand control to the caller via `ui_logic` to then start the ui runtime.
///
/// This should **NOT** be called if `ui_logic` will start an async runtime.
pub fn koru_main_ui_start_runtime<F>(ui_logic: F) -> Result<(), Box<dyn Error>>
where F: FnOnce(Sender<ClientConnectingMessage>, Receiver<ClientConnectingResponse>) -> Result<(), Box<dyn Error>>
{
    kernel::start_kernel(ui_logic)
}

