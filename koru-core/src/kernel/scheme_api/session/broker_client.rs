use std::collections::HashMap;
use std::error::Error;
use std::sync::{LazyLock, RwLock, RwLockReadGuard, RwLockWriteGuard};
use guile_rs::scheme_object::{SchemeObject, SchemeSmob};
use guile_rs::{guile_misc_error, guile_wrong_type_arg, SchemeValue};
use crate::kernel::broker::{BrokerClient, Message, MessageKind, MESSAGE_KIND_SMOB_TAG, MESSAGE_SMOB_TAG};
use crate::kernel::scheme_api::session::get_session_id;
use crate::kernel::session::SessionId;

pub static SESSION_BROKER_CLIENTS: LazyLock<RwLock<HashMap<SessionId, SchemeSmob<BrokerClient>>>> = LazyLock::new(|| {
    RwLock::new(HashMap::new())
});

pub fn get_broker_clients() -> RwLockReadGuard<'static, HashMap<SessionId, SchemeSmob<BrokerClient>>> {
    let Ok(guard) = SESSION_BROKER_CLIENTS.read() else {
        panic!("Lock poisoned");
    };
    guard
}

pub fn get_broker_clients_mut() -> RwLockWriteGuard<'static, HashMap<SessionId, SchemeSmob<BrokerClient>>> {
    let Ok(guard) = SESSION_BROKER_CLIENTS.write() else {
        panic!("Lock poisoned");
    };
    guard
}

pub extern "C" fn send_message_scheme(message: SchemeValue, destination: SchemeValue) -> SchemeValue {
    let Some(message) = SchemeObject::from(message).cast_smob(MESSAGE_KIND_SMOB_TAG.clone()) else {
        guile_wrong_type_arg!("send-message", 1, message);
    };
    let Some(destination) = SchemeObject::from(destination).cast_number() else {
        guile_wrong_type_arg!("send-message", 2, destination);
    };
    let destination = destination.as_u64() as usize;
    
    send_message(message.borrow().clone(), destination);
    
    SchemeValue::undefined()
}

pub fn send_message(message: MessageKind, destination: usize) {
    let session_id = get_session_id();
    let broker_clients = get_broker_clients();
    let client = broker_clients.get(&session_id).unwrap();
    match client.borrow_mut().send(message, destination) {
        Ok(_) => {}
        Err(e) => {
            panic!("{}", e);
        }
    }
}

pub extern "C" fn recv_message_scheme() -> SchemeValue {
    match recv_message() {
        Some(message) => {
            <SchemeSmob<Message> as Into<SchemeObject>>::into(MESSAGE_SMOB_TAG.make(message)).into()
        }
        None => {
            guile_misc_error!("recv-message", "sender died");
        }
    }
}

pub fn recv_message() -> Option<Message> {
    let session_id = get_session_id();
    let broker_clients = get_broker_clients();
    let client = broker_clients.get(&session_id).unwrap();
    client.borrow_mut().recv()
}

pub extern "C" fn send_response_scheme(message: SchemeValue, mail: SchemeValue) -> SchemeValue {
    let Some(message) = SchemeObject::from(message).cast_smob(MESSAGE_KIND_SMOB_TAG.clone()) else {
        guile_wrong_type_arg!("send-response", 1, message);
    };
    let Some(mail) = SchemeObject::from(mail).cast_smob(MESSAGE_SMOB_TAG.clone()) else {
        guile_wrong_type_arg!("send-response", 2, mail);
    };
    match send_response(message.borrow().clone(), mail.borrow().clone()) {
        Ok(_) => {}
        Err(e) => {
            panic!("{}", e);
        }
    }
    SchemeValue::undefined()
}

pub fn send_response(message: MessageKind, mail: Message) -> Result<(), Box<dyn Error>> {
    let session_id = get_session_id();
    let broker_clients = get_broker_clients();
    let client = broker_clients.get(&session_id).unwrap();
    
    client.borrow_mut().send_response(message, mail)
}