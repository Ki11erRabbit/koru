use std::collections::HashMap;
use std::sync::{LazyLock, RwLock, RwLockReadGuard, RwLockWriteGuard};
use guile_rs::scheme_object::{SchemeList, SchemeObject, SchemeProcedure};
use guile_rs::{guile_wrong_type_arg, SchemeValue};
use crate::kernel::scheme_api::session::get_session_id;
use crate::kernel::session::SessionId;

pub static SESSION_HOOKS: LazyLock<RwLock<HashMap<SessionId, HashMap<String, HashMap<String, SchemeProcedure>>>>> = LazyLock::new(|| {
    RwLock::new(HashMap::new())
});

pub fn get_session_hooks() -> RwLockReadGuard<'static, HashMap<SessionId, HashMap<String, HashMap<String, SchemeProcedure>>>> {
    let Ok(guard) = SESSION_HOOKS.read() else {
        panic!("Lock poisoned");
    };
    guard
}

pub fn get_session_hooks_mut() -> RwLockWriteGuard<'static, HashMap<SessionId, HashMap<String, HashMap<String, SchemeProcedure>>>> {
    let Ok(guard) = SESSION_HOOKS.write() else {
        panic!("Lock poisoned");
    };
    guard
}

pub extern "C" fn create_hook(hook_name: SchemeValue) -> SchemeValue {
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

pub extern "C" fn add_hook(hook_name: SchemeValue, proc_name: SchemeValue, function: SchemeValue) -> SchemeValue {
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

pub extern "C" fn remove_hook(hook_name: SchemeValue, proc_name: SchemeValue) -> SchemeValue {
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

pub extern "C" fn call_hook(hook_name: SchemeValue, rest: SchemeValue) -> SchemeValue {
    let Some(hook_name) = SchemeObject::from(hook_name).cast_symbol() else {
        guile_wrong_type_arg!("call-hook", 1, hook_name);
    };
    let Some(rest) = SchemeObject::from(rest).cast_list() else {
        guile_wrong_type_arg!("call-hook", 2, rest);
    };
    
    call_hooks(&hook_name.to_string(), rest);

    SchemeValue::undefined()
}

pub fn call_hooks(hook_name: &str, rest: SchemeList) {
    let session_hooks = get_session_hooks_mut();
    let session_id = get_session_id();
    let Some(hooks) = session_hooks.get(&session_id) else {
        panic!("Session ID not found");
    };
    if let Some(hooks) = hooks.get(hook_name) {
        for (_, proc) in hooks.iter() {
            let rest = rest.clone().iter().collect::<Vec<_>>();
            proc.call(rest);
        }
    }
}