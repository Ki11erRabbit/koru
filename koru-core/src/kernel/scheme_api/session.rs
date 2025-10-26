use std::sync::LazyLock;
use guile_rs::{Guile, Module, SchemeValue};
use guile_rs::fluid::{Fluid, FluidId};
use guile_rs::scheme_object::SchemeObject;
use crate::kernel::session::SessionId;

static SESSION_ID_FLUID: LazyLock<FluidId> = LazyLock::new(|| {
    Fluid::make_default(SchemeObject::from(-1))
});

extern "C" fn get_session_id() -> SchemeValue {
    let value = Fluid::get(SESSION_ID_FLUID.clone());
    value.into()
}

pub fn set_session_id(session_id: SessionId) {
    Fluid::set(SESSION_ID_FLUID.clone(), SchemeObject::from(session_id.get()));
}

pub fn koru_session_module() {
    Guile::define_fn("get-session-id", 0, 0, false,
        get_session_id as extern "C" fn() -> SchemeValue      
    );
    let mut module = Module::new("koru-session", Box::new(|_| {}));
    module.add_export("get-session-id");
    module.export();
    module.define(&mut ());
}