use std::collections::HashMap;
use std::sync::Arc;
use scheme_rs::exceptions::Condition;
use scheme_rs::gc::{Gc, Trace};
use scheme_rs::records::{rtd, Record, RecordTypeDescriptor, SchemeCompatible};
use scheme_rs::registry::bridge;
use scheme_rs::value::Value;
use tokio::sync::RwLock;
use crate::kernel::input::KeyPress;
use crate::kernel::scheme_api::command::Command;
use crate::keymap::KeyMap;

#[derive(Debug, Trace)]
pub struct SchemeKeyMap {
    mapping: RwLock<HashMap<String, Gc<Command>>>,
    default: Option<Gc<Command>>,
}

impl SchemeKeyMap {
    pub fn new(default: Option<Gc<Command>>) -> Self {
        Self {
            mapping: RwLock::new(HashMap::new()),
            default
        }
    }

    pub async fn add_binding(&self, key: &str, command: Gc<Command>) {
        self.mapping.write().await.insert(key.to_string(), command);
    }

    pub async fn remove_binding(&self, key: &str) {
        self.mapping.write().await.remove(key);
    }

    pub async fn make_keymap(&self) -> Result<KeyMap, String> {
        let mut guard = self.mapping.write().await;
        let mut key_map = KeyMap::new_sparse();
        for (key, value) in guard.drain() {
            let vec = key.split_whitespace()
                .map(|s| {
                    KeyPress::from_string(s)
                })
                .collect::<Option<Vec<KeyPress>>>();
            let Some(vec) = vec else {
                return Err(String::from("Invalid key sequence."));
            };
            key_map.add_binding(vec, value);
        }
        if let Some(default) = &self.default {
            key_map.set_default(default.clone());
        }
        Ok(key_map)
    }
}

impl SchemeCompatible for SchemeKeyMap {
    fn rtd() -> Arc<RecordTypeDescriptor>
    where
        Self: Sized
    {
        rtd!(name: "&KeyMap")
    }
}

#[bridge(name = "key-map-create", lib = "(key-map)")]
pub fn create_keymap(default: &[Value]) -> Result<Vec<Value>, Condition> {
    let Some((default, _)) = default.split_first() else {
        return Ok(vec![Value::from(Record::from_rust_type(SchemeKeyMap::new(None)))])
    };
    let default: Gc<Command> = default.clone().try_into_rust_type()?;

    Ok(vec![Value::from(Record::from_rust_type(SchemeKeyMap::new(Some(default))))])
}

#[bridge(name = "key-map-insert", lib = "(key-map)")]
pub async fn keymap_insert(args: &[Value]) -> Result<Vec<Value>, Condition> {
    let Some((keymap, rest)) = args.split_first() else {
        return Err(Condition::wrong_num_of_args(3, args.len()));
    };
    let Some((key_sequence, rest)) = rest.split_first() else {
        return Err(Condition::wrong_num_of_args(3, args.len()));
    };
    let Some((command, _)) = rest.split_first() else {
        return Err(Condition::wrong_num_of_args(3, args.len()));
    };
    let keymap: Gc<SchemeKeyMap> = keymap.clone().try_into_rust_type()?;
    let key_sequence: String = key_sequence.clone().try_into()?;
    let command: Gc<Command> = command.clone().try_into_rust_type()?;

    keymap.add_binding(&key_sequence, command).await;
    Ok(vec![])
}

#[bridge(name = "key-map-delete", lib = "(key-map)")]
pub async fn keymap_delete(args: &[Value]) -> Result<Vec<Value>, Condition> {
    let Some((keymap, rest)) = args.split_first() else {
        return Err(Condition::wrong_num_of_args(2, args.len()));
    };
    let Some((key_sequence, _)) = rest.split_first() else {
        return Err(Condition::wrong_num_of_args(2, args.len()));
    };
    let keymap: Gc<SchemeKeyMap> = keymap.clone().try_into_rust_type()?;
    let key_sequence: String = key_sequence.clone().try_into()?;

    keymap.remove_binding(&key_sequence).await;
    Ok(vec![])
}