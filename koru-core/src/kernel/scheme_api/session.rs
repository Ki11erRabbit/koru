use guile_rs::{Guile, Module, SchemeValue};
use guile_rs::scheme_object::SchemeObject;
use crate::kernel::session::SessionId;

extern "C" fn get_session_id() -> SchemeValue {
    let Some(id) = Guile::module_lookup("koru-session", "session-id") else {
        return SchemeObject::undefined().into();
    };
    id.into()
}

pub fn koru_session_module(session_id: SessionId) {
    Guile::define("session-id", <usize as Into<SchemeObject>>::into(session_id.get()));
    
    let mut module = Module::new("koru-session", Box::new(|_| {}));
    module.add_export("get-session-id");
    module.export();
    module.define(&mut ());
}