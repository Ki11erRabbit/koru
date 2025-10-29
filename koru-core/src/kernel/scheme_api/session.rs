use std::collections::HashMap;
use std::sync::{LazyLock, RwLock, RwLockReadGuard, RwLockWriteGuard};
use guile_rs::{guile_misc_error, guile_wrong_type_arg, Guile, Module, SchemeValue};
use guile_rs::fluid::{Fluid, FluidId};
use guile_rs::scheme_object::{SchemeObject, SchemeProcedure, SchemeSmob};
use crate::kernel::broker::{BrokerClient, Message, BROKER_CLIENT_SMOB_TAG, MESSAGE_KIND_SMOB_TAG, MESSAGE_SMOB_TAG};
use crate::kernel::session::SessionId;

static SESSION_ID_FLUID: LazyLock<FluidId> = LazyLock::new(|| {
    Fluid::make_default(SchemeObject::from(-1))
});

static SESSION_HOOKS: LazyLock<RwLock<HashMap<SessionId, HashMap<String, HashMap<String, SchemeProcedure>>>>> = LazyLock::new(|| {
    RwLock::new(HashMap::new())
});

fn get_session_hooks() -> RwLockReadGuard<'static, HashMap<SessionId, HashMap<String, HashMap<String, SchemeProcedure>>>> {
    let Ok(guard) = SESSION_HOOKS.read() else {
        panic!("Lock poisoned");
    };
    guard
}

fn get_session_hooks_mut() -> RwLockWriteGuard<'static, HashMap<SessionId, HashMap<String, HashMap<String, SchemeProcedure>>>> {
    let Ok(guard) = SESSION_HOOKS.write() else {
        panic!("Lock poisoned");
    };
    guard
}


static SESSION_BROKER_CLIENTS: LazyLock<RwLock<HashMap<SessionId, SchemeSmob<BrokerClient>>>> = LazyLock::new(|| {
    RwLock::new(HashMap::new())
});

fn get_broker_clients() -> RwLockReadGuard<'static, HashMap<SessionId, SchemeSmob<BrokerClient>>> {
    let Ok(guard) = SESSION_BROKER_CLIENTS.read() else {
        panic!("Lock poisoned");
    };
    guard
}

fn get_broker_clients_mut() -> RwLockWriteGuard<'static, HashMap<SessionId, SchemeSmob<BrokerClient>>> {
    let Ok(guard) = SESSION_BROKER_CLIENTS.write() else {
        panic!("Lock poisoned");
    };
    guard
}


extern "C" fn get_session_id_scheme() -> SchemeValue {
    let value = Fluid::get(SESSION_ID_FLUID.clone());
    value.into()
}

fn get_session_id() -> SessionId {
    let Some(id) = SchemeObject::from(get_session_id_scheme()).cast_number() else {
        panic!("Failed to convert number to SessionId");
    };
    SessionId::new(id.as_u64() as usize)
}

pub fn set_session_id(session_id: SessionId) {
    let mut session_hooks = get_session_hooks_mut();
    session_hooks.insert(session_id, HashMap::new());

    Fluid::set(SESSION_ID_FLUID.clone(), SchemeObject::from(session_id.get()));
}

pub fn set_broker_client(session_id: SessionId, client: BrokerClient) {
    let mut session_broker_clients = get_broker_clients_mut();
    let client = BROKER_CLIENT_SMOB_TAG.make(client);
    session_broker_clients.insert(session_id, client);
}

extern "C" fn send_message(message: SchemeValue, destination: SchemeValue) -> SchemeValue {
    let session_id = get_session_id();
    let broker_clients = get_broker_clients();
    let client = broker_clients.get(&session_id).unwrap();
    
    let Some(message) = SchemeObject::from(message).cast_smob(MESSAGE_KIND_SMOB_TAG.clone()) else {
        guile_wrong_type_arg!("send-message", 1, message);
    };
    let Some(destination) = SchemeObject::from(destination).cast_number() else {
        guile_wrong_type_arg!("send-message", 2, destination);
    };
    let destination = destination.as_u64() as usize;
    match client.borrow_mut().send(message.borrow().clone(), destination) {
        Ok(_) => {}
        Err(e) => {
            panic!("{}", e);
        }
    }
    SchemeValue::undefined()
}

extern "C" fn recv_message() -> SchemeValue {
    let session_id = get_session_id();
    let broker_clients = get_broker_clients();
    let client = broker_clients.get(&session_id).unwrap();

    match client.borrow_mut().recv() {
        Some(message) => {
            <SchemeSmob<Message> as Into<SchemeObject>>::into(MESSAGE_SMOB_TAG.make(message)).into()
        }
        None => {
            guile_misc_error!("recv-message", "sender died");
        }
    }
}

extern "C" fn send_response(message: SchemeValue, mail: SchemeValue) -> SchemeValue {
    let session_id = get_session_id();
    let broker_clients = get_broker_clients();
    let client = broker_clients.get(&session_id).unwrap();

    let Some(message) = SchemeObject::from(message).cast_smob(MESSAGE_KIND_SMOB_TAG.clone()) else {
        guile_wrong_type_arg!("send-response", 1, message);
    };
    let Some(mail) = SchemeObject::from(mail).cast_smob(MESSAGE_SMOB_TAG.clone()) else {
        guile_wrong_type_arg!("send-response", 2, mail);
    };
    match client.borrow_mut().send_response(message.borrow().clone(), mail.borrow().clone()) {
        Ok(_) => {}
        Err(e) => {
            panic!("{}", e);
        }
    }
    SchemeValue::undefined()
}

extern "C" fn create_hook(hook_name: SchemeValue) -> SchemeValue {
    let Some(hook_name) = SchemeObject::from(hook_name).cast_symbol() else {
        guile_wrong_type_arg!("create-hook", 1, hook_name);
    };
    let mut session_hooks = get_session_hooks_mut();
    let session_id = get_session_id();

    session_hooks.entry(session_id)
        .and_modify(|session_hooks| {
            session_hooks.insert(hook_name.to_string(), HashMap::new());
        })
        .or_insert_with(HashMap::new);

    SchemeValue::undefined()
}

extern "C" fn add_hook(hook_name: SchemeValue, proc_name: SchemeValue, function: SchemeValue) -> SchemeValue {
    let Some(hook_name) = SchemeObject::from(hook_name).cast_symbol() else {
        guile_wrong_type_arg!("add-hook", 1, hook_name);
    };
    let Some(proc_name) = SchemeObject::from(proc_name).cast_symbol() else {
        guile_wrong_type_arg!("add-hook", 2, proc_name);
    };
    let Some(function) = SchemeObject::from(function).cast_procedure() else {
        guile_wrong_type_arg!("add-hook", 3, function);
    };
    let mut session_hooks = get_session_hooks_mut();
    let session_id = get_session_id();

    let mut default = HashMap::new();
    default.insert(proc_name.to_string(), function.clone());

    session_hooks.entry(session_id)
        .and_modify(|session_hooks| {
            session_hooks.entry(hook_name.to_string())
                .and_modify(|hooks| {
                    hooks.insert(proc_name.to_string(), function.clone());
                })
                .or_insert(default);
        })
        .or_insert_with(HashMap::new);

    SchemeValue::undefined()
}

extern "C" fn remove_hook(hook_name: SchemeValue, proc_name: SchemeValue) -> SchemeValue {
    let Some(hook_name) = SchemeObject::from(hook_name).cast_symbol() else {
        guile_wrong_type_arg!("remove-hook", 1, hook_name);
    };
    let Some(proc_name) = SchemeObject::from(proc_name).cast_symbol() else {
        guile_wrong_type_arg!("remove-hook", 2, proc_name);
    };
    let mut session_hooks = get_session_hooks_mut();
    let session_id = get_session_id();
    session_hooks.entry(session_id)
    .and_modify(|session_hooks| {
        session_hooks.entry(hook_name.to_string())
        .and_modify(|hooks| {
            hooks.remove(&proc_name.to_string());
        });
    });
    SchemeValue::undefined()
}

extern "C" fn call_hook(hook_name: SchemeValue, rest: SchemeValue) -> SchemeValue {
    let Some(hook_name) = SchemeObject::from(hook_name).cast_string() else {
        guile_wrong_type_arg!("call-hook", 1, hook_name);
    };
    let Some(rest) = SchemeObject::from(rest).cast_list() else {
        guile_wrong_type_arg!("call-hook", 2, rest);
    };

    let mut session_hooks = get_session_hooks_mut();
    let session_id = get_session_id();
    let Some(hooks) = session_hooks.get(&session_id) else {
        panic!("Session ID not found");
    };
    if let Some(hooks) = hooks.get(&hook_name.to_string()) {
        for (_, proc) in hooks.iter() {
            let rest = rest.clone().iter().collect::<Vec<_>>();
            proc.call(rest);
        }
    }

    SchemeValue::undefined()
}

pub fn koru_session_module() {
    Guile::define_fn("get-session-id", 0, 0, false,
                     get_session_id_scheme as extern "C" fn() -> SchemeValue
    );
    Guile::define_fn("create-hook", 1, 0, false,
                     create_hook as extern "C" fn(SchemeValue) -> SchemeValue
    );
    Guile::define_fn("add-hook", 3, 0, false,
                     add_hook as extern "C" fn(SchemeValue, SchemeValue, SchemeValue) -> SchemeValue
    );
    Guile::define_fn("remove-hook", 2, 0, false,
                     remove_hook as extern "C" fn(SchemeValue, SchemeValue) -> SchemeValue
    );
    Guile::define_fn("call-hook", 1, 0, true,
                     call_hook as extern "C" fn(SchemeValue, SchemeValue) -> SchemeValue
    );
    Guile::define_fn("send-message", 2, 0, false,
        send_message as extern "C" fn(SchemeValue, SchemeValue) -> SchemeValue
    );
    Guile::define_fn("recv-message", 0, 0, false,
        recv_message as extern "C" fn() -> SchemeValue
    );
    Guile::define_fn("send-response", 2, 0, false,
                     send_response as extern "C" fn(SchemeValue, SchemeValue) -> SchemeValue
    );
    let mut module = Module::new("koru-session", Box::new(|_| {}));
    module.add_export("get-session-id");
    module.add_export("create-hook");
    module.add_export("add-hook");
    module.add_export("remove-hook");
    module.add_export("call-hook");
    module.export();
    module.define(&mut ());
}