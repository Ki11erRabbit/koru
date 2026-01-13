use std::sync::Arc;
use scheme_rs::exceptions::Exception;
use scheme_rs::gc::{Gc, Trace};
use scheme_rs::proc::Procedure;
use scheme_rs::records::{rtd, Record, RecordTypeDescriptor, SchemeCompatible};
use scheme_rs::registry::bridge;
use scheme_rs::symbols::Symbol;
use scheme_rs::value::Value;
use tokio::sync::RwLock;
use crate::kernel::scheme_api::session::SessionState;

#[derive(Debug)]
struct ModalInternal {
    /// The current state the mode is in
    state: Symbol,
    /// The hook to be emitted when the state changes.
    state_change_hook_name: Symbol,
    /// A function to be called when the state changes.
    state_change_callback: Value,
    /// The prefix to use with the `command-bar-update` function
    prefix: String,
    /// The suffix to use with the `command-bar-update` function
    suffix: String,
    /// The callback to use when leaving command entry
    command_callback: Value,
}

impl ModalInternal {
    pub fn new(
        state: Symbol,
        state_change_hook_name: Symbol,
        state_change_callback: Value,
    ) -> Self {
        Self {
            state,
            state_change_hook_name,
            state_change_callback,
            prefix: String::new(),
            suffix: String::new(),
            command_callback: Value::null(),
        }
    }
}

#[derive(Debug, Trace)]
pub struct Modal {
    internal: Arc<RwLock<ModalInternal>>
}

impl Modal {
    pub fn new(
        initial_state: Symbol,
        hook_name: Symbol,
        callback: Value,
    ) -> Self {
        let internal = ModalInternal::new(initial_state, hook_name, callback);
        let internal = Arc::new(RwLock::new(internal));
        Self {
            internal
        }
    }

    pub async fn change_state(&self, state: Symbol) -> Result<(), Exception> {
        let (old_state, hook_name, callback) = {
            let mut guard = self.internal.write().await;
            let old_state = guard.state;
            let state_change_hook_name = guard.state_change_hook_name.clone();
            let state_change_callback = guard.state_change_callback.clone();
            guard.state = state;
            (old_state, state_change_hook_name, state_change_callback)
        };

        if old_state == state {
            return Ok(())
        }

        let callback: Procedure = callback.try_into()?;
        callback.call(&[Value::from(old_state.clone()), Value::from(state.clone())]).await?;

        SessionState::emit_hook(
            hook_name,
            &[Value::from(old_state), Value::from(hook_name.clone())]
        ).await?;

        Ok(())
    }

    pub async fn get_state(&self) -> Symbol {
        self.internal.read().await.state.clone()
    }

    pub async fn set_command_callback(&self, callback: Value) {
        let mut guard = self.internal.write().await;
        guard.command_callback = callback;
    }

    pub async fn get_command_callback(&self) -> Value {
        self.internal.read().await.command_callback.clone()
    }

    pub async fn set_prefix(&self, prefix: String) {
        let mut guard = self.internal.write().await;
        guard.prefix = prefix;
    }

    pub async fn get_prefix(&self) -> String {
        self.internal.read().await.prefix.clone()
    }

    pub async fn set_suffix(&self, suffix: String) {
        let mut guard = self.internal.write().await;
        guard.suffix = suffix;
    }

    pub async fn get_suffix(&self) -> String {
        self.internal.read().await.suffix.clone()
    }
}

impl SchemeCompatible for Modal {
    fn rtd() -> Arc<RecordTypeDescriptor>
    where
        Self: Sized
    {
        rtd!(name: "&Modal", sealed: true)
    }
}

#[bridge(name = "modal-create", lib = "(koru-modal)")]
pub async fn modal_create(args: &[Value]) -> Result<Vec<Value>, Exception> {
    let Some((initial_state, rest)) = args.split_first() else {
        return Err(Exception::wrong_num_of_args(3, args.len()))
    };
    let Some((hook_name, rest)) = rest.split_first() else {
        return Err(Exception::wrong_num_of_args(3, args.len()))
    };
    let Some((callback, _)) = rest.split_first() else {
        return Err(Exception::wrong_num_of_args(3, args.len()))
    };
    let initial_state: Symbol = initial_state.clone().try_into()?;
    let hook_name: Symbol = hook_name.clone().try_into()?;
    let _callback: Procedure = callback.clone().try_into()?;

    let modal = Modal::new(initial_state, hook_name, callback.clone());
    let modal = Record::from_rust_type(modal);

    Ok(vec![Value::from(modal)])
}

#[bridge(name = "modal-state-set!", lib = "(koru-modal)")]
pub async fn modal_change_state(args: &[Value]) -> Result<Vec<Value>, Exception> {
    let Some((modal, rest)) = args.split_first() else {
        return Err(Exception::wrong_num_of_args(2, args.len()))
    };
    let Some((state, _)) = rest.split_first() else {
        return Err(Exception::wrong_num_of_args(2, args.len()))
    };
    let modal: Gc<Modal> = modal.try_to_rust_type()?;
    let state: Symbol = state.clone().try_into()?;

    modal.change_state(state).await?;

    Ok(Vec::new())
}

#[bridge(name = "modal-state", lib = "(koru-modal)")]
pub async fn modal_state(modal: &Value) -> Result<Vec<Value>, Exception> {
    let modal: Gc<Modal> = modal.try_to_rust_type()?;
    let output = modal.get_state().await;
    Ok(vec![Value::from(output)])
}

#[bridge(name = "modal-prefix-set!", lib = "(koru-modal)")]
pub async fn modal_change_prefix(args: &[Value]) -> Result<Vec<Value>, Exception> {
    let Some((modal, rest)) = args.split_first() else {
        return Err(Exception::wrong_num_of_args(2, args.len()))
    };
    let Some((prefix, _)) = rest.split_first() else {
        return Err(Exception::wrong_num_of_args(2, args.len()))
    };
    let modal: Gc<Modal> = modal.try_to_rust_type()?;
    let prefix: String = prefix.clone().try_into()?;

    modal.set_prefix(prefix).await;

    Ok(Vec::new())
}

#[bridge(name = "modal-prefix", lib = "(koru-modal)")]
pub async fn modal_prefix(modal: &Value) -> Result<Vec<Value>, Exception> {
    let modal: Gc<Modal> = modal.try_to_rust_type()?;
    let output = modal.get_prefix().await;
    Ok(vec![Value::from(output)])
}

#[bridge(name = "modal-suffix-set!", lib = "(koru-modal)")]
pub async fn modal_change_suffix(args: &[Value]) -> Result<Vec<Value>, Exception> {
    let Some((modal, rest)) = args.split_first() else {
        return Err(Exception::wrong_num_of_args(2, args.len()))
    };
    let Some((suffix, _)) = rest.split_first() else {
        return Err(Exception::wrong_num_of_args(2, args.len()))
    };
    let modal: Gc<Modal> = modal.try_to_rust_type()?;
    let suffix: String = suffix.clone().try_into()?;

    modal.set_suffix(suffix).await;

    Ok(Vec::new())
}

#[bridge(name = "modal-suffix", lib = "(koru-modal)")]
pub async fn suffix(modal: &Value) -> Result<Vec<Value>, Exception> {
    let modal: Gc<Modal> = modal.try_to_rust_type()?;
    let output = modal.get_suffix().await;
    Ok(vec![Value::from(output)])
}

#[bridge(name = "modal-callback-set!", lib = "(koru-modal)")]
pub async fn modal_change_callback(args: &[Value]) -> Result<Vec<Value>, Exception> {
    let Some((modal, rest)) = args.split_first() else {
        return Err(Exception::wrong_num_of_args(2, args.len()))
    };
    let Some((callback, _)) = rest.split_first() else {
        return Err(Exception::wrong_num_of_args(2, args.len()))
    };
    let modal: Gc<Modal> = modal.try_to_rust_type()?;

    // We let the user null out the field if they want to
    // The if the value isn't null, then we ensure that it is a procedure.
    if !callback.is_null() {
        let _callback: Procedure = callback.clone().try_into()?;
    }

    modal.set_command_callback(callback.clone()).await;

    Ok(Vec::new())
}

#[bridge(name = "modal-callback-apply", lib = "(koru-modal)")]
pub async fn callback_apply(modal: &Value) -> Result<Vec<Value>, Exception> {
    let modal: Gc<Modal> = modal.try_to_rust_type()?;
    let callback = modal.get_command_callback().await;
    if !callback.is_null() {
        let callback: Procedure = callback.clone().try_into()?;
        callback.call(&[]).await?;
    }
    Ok(Vec::new())
}