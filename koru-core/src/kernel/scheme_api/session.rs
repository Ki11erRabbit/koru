use std::collections::HashMap;
use std::sync::{LazyLock, RwLock, RwLockReadGuard, RwLockWriteGuard};
use guile_rs::{Guile, Module, SchemeValue};
use guile_rs::fluid::{Fluid, FluidId};
use guile_rs::scheme_object::{SchemeObject, SchemeProcedure};
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

extern "C" fn get_session_id_scheme() -> SchemeValue {
    let value = Fluid::get(SESSION_ID_FLUID.clone());
    value.into()
}

fn get_session_id() -> SessionId {
    let Some(id) = SchemeObject::new(get_session_id_scheme()).cast_number() else {
        panic!("Failed to convert number to SessionId");
    };
    SessionId::new(id.as_u64() as usize)
}

pub fn set_session_id(session_id: SessionId) {
    let mut session_hooks = get_session_hooks_mut();
    session_hooks.insert(session_id, HashMap::new());
    
    Fluid::set(SESSION_ID_FLUID.clone(), SchemeObject::from(session_id.get()));
}

extern "C" fn create_hook(hook_name: SchemeValue) -> SchemeValue {
    let Some(hook_name) = SchemeObject::new(hook_name).cast_string() else {
        return SchemeObject::undefined().into();
    };
    let mut session_hooks = get_session_hooks_mut();
    let session_id = get_session_id();
    
    session_hooks.entry(session_id)
        .and_modify(|session_hooks| {
            session_hooks.insert(hook_name.to_string(), HashMap::new());
        })
        .or_insert_with(HashMap::new);
    
    SchemeObject::undefined().into()
}

extern "C" fn add_hook(hook_name: SchemeValue, proc_name: SchemeValue, function: SchemeValue) -> SchemeValue {
    let Some(hook_name) = SchemeObject::new(hook_name).cast_string() else {
        return SchemeObject::undefined().into();
    };
    let Some(proc_name) = SchemeObject::new(proc_name).cast_string() else {
        return SchemeObject::undefined().into();
    };
    let Some(function) = SchemeObject::new(function).cast_procedure() else {
        return SchemeObject::undefined().into();
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

    SchemeObject::undefined().into()
}

extern "C" fn remove_hook(hook_name: SchemeValue, proc_name: SchemeValue) -> SchemeValue {
    let Some(hook_name) = SchemeObject::new(hook_name).cast_string() else {
        return SchemeObject::undefined().into();
    };
    let Some(proc_name) = SchemeObject::new(proc_name).cast_string() else {
        return SchemeObject::undefined().into();
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
    SchemeObject::undefined().into()
}

extern "C" fn call_hook(hook_name: SchemeValue, rest: SchemeValue) -> SchemeValue {
    let Some(hook_name) = SchemeObject::new(hook_name).cast_string() else {
        return SchemeObject::undefined().into();
    };
    let Some(rest) = SchemeObject::new(rest).cast_list() else {
        return SchemeObject::undefined().into();
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
    
    SchemeObject::undefined().into()
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
    let mut module = Module::new("koru-session", Box::new(|_| {}));
    module.add_export("get-session-id");
    module.add_export("create-hook");
    module.add_export("add-hook");
    module.add_export("remove-hook");
    module.add_export("call-hook");
    module.export();
    module.define(&mut ());
}