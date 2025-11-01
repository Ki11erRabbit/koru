
use std::collections::HashMap;
use std::sync::{LazyLock, };
use crate::kernel::broker::{BrokerClient};
use crate::kernel::session::SessionId;

/*
static SESSION_ID_FLUID: LazyLock<FluidId> = LazyLock::new(|| {
    Fluid::make_default(SchemeObject::from(-1))
});

extern "C" fn get_session_id_scheme() -> SchemeValue {
    let value = Fluid::get(SESSION_ID_FLUID.clone());
    value.into()
}

pub fn get_session_id() -> SessionId {
    let Some(id) = SchemeObject::from(get_session_id_scheme()).cast_number() else {
        panic!("Failed to convert number to SessionId");
    };
    SessionId::new(id.as_u64() as usize)
}

pub fn set_session_id(session_id: SessionId, client: BrokerClient) {
    let mut session_hooks = hooks::get_session_hooks_mut();
    session_hooks.insert(session_id, HashMap::new());
    let mut session_broker_clients = get_broker_clients_mut();
    let client = BROKER_CLIENT_SMOB_TAG.make(client);
    session_broker_clients.insert(session_id, client);

    Fluid::set(SESSION_ID_FLUID.clone(), SchemeObject::from(session_id.get()));
}

pub fn koru_session_module() {
    Guile::define_fn("get-session-id", 0, 0, false,
                     get_session_id_scheme as extern "C" fn() -> SchemeValue
    );
    Guile::define_fn("create-hook", 1, 0, false,
                     hooks::create_hook as extern "C" fn(SchemeValue) -> SchemeValue
    );
    Guile::define_fn("add-hook", 3, 0, false,
                     hooks::add_hook as extern "C" fn(SchemeValue, SchemeValue, SchemeValue) -> SchemeValue
    );
    Guile::define_fn("remove-hook", 2, 0, false,
                     hooks::remove_hook as extern "C" fn(SchemeValue, SchemeValue) -> SchemeValue
    );
    Guile::define_fn("call-hook", 1, 0, true,
                     hooks::call_hook as extern "C" fn(SchemeValue, SchemeValue) -> SchemeValue
    );
    Guile::define_fn("send-message", 2, 0, false,
        broker_client::send_message_scheme as extern "C" fn(SchemeValue, SchemeValue) -> SchemeValue
    );
    Guile::define_fn("recv-message", 0, 0, false,
                     broker_client::recv_message_scheme as extern "C" fn() -> SchemeValue
    );
    Guile::define_fn("send-response", 2, 0, false,
                     broker_client::send_response_scheme as extern "C" fn(SchemeValue, SchemeValue) -> SchemeValue
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
*/