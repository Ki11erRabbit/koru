use koru_core::kernel::broker::{BrokerClient, Message};
use koru_core::kernel::input::KeyPress;

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