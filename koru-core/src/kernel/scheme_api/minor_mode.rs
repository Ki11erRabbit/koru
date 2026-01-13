use std::collections::{HashMap, VecDeque};
use std::sync::Arc;
use scheme_rs::exceptions::Exception;
use scheme_rs::gc::{Gc, Trace};
use scheme_rs::proc::Procedure;
use scheme_rs::records::{rtd, Record, RecordTypeDescriptor, SchemeCompatible};
use scheme_rs::registry::bridge;
use scheme_rs::symbols::Symbol;
use scheme_rs::value::Value;
use tokio::sync::RwLock;
use crate::kernel::scheme_api::major_mode::MajorMode;

struct MinorModeManagerMetadata {
    map: HashMap<Symbol, usize>,
    free_list: VecDeque<usize>,
}

impl MinorModeManagerMetadata {
    pub fn new() -> Self {
        MinorModeManagerMetadata {
            map: HashMap::new(),
            free_list: VecDeque::new(),
        }
    }
}

#[derive(Clone)]
pub struct MinorModeManager {
    minor_modes: Vec<Option<Value>>,
    metadata: Arc<RwLock<MinorModeManagerMetadata>>,
}

impl MinorModeManager {
    pub fn new() -> Self {
        Self {
            minor_modes: Vec::new(),
            metadata: Arc::new(RwLock::new(MinorModeManagerMetadata::new())),
        }
    }

    pub async fn add_minor_mode(&mut self, minor_mode: Value) -> Result<(), Exception> {
        let minor_mode_value: Gc<MinorMode> = minor_mode.try_to_rust_type()?;
        let mut guard = self.metadata.write().await;
        if guard.map.contains_key(&minor_mode_value.name) {
            return Ok(());
        }
        {
            let mm: Gc<MinorMode> = minor_mode.try_to_rust_type()?;
            let gain_focus = mm.gain_focus();
            gain_focus.call(&[minor_mode.clone()]).await?;
        }
        if let Some(index) = guard.free_list.pop_front() {
            self.minor_modes[index] = Some(minor_mode);
        } else {
            let index = guard.free_list.len();
            let name = minor_mode_value.name.clone();
            guard.map.insert(name, index);
            self.minor_modes.push(Some(minor_mode));
        }
        Ok(())
    }

    pub async fn remove_minor_mode(&mut self, minor_mode_name: Symbol) -> Option<String> {
        let mut guard = self.metadata.write().await;
        if let Some(index) = guard.map.remove(&minor_mode_name) {
            self.minor_modes[index] = None;
            guard.free_list.push_back(index);
            Some(minor_mode_name.to_string())
        } else {
            None
        }
    }

    pub async fn get_minor_mode(&self, minor_mode_name: Symbol) -> Option<&Value> {
        let guard = self.metadata.read().await;
        if let Some(index) = guard.map.get(&minor_mode_name) {
            self.minor_modes[*index].as_ref()
        } else {
            None
        }
    }

    pub fn get_minor_modes(&self) -> Vec<Value> {
        let mut minor_modes = Vec::new();
        for mode in self.minor_modes.iter() {
            if let Some(minor_mode) = mode {
                minor_modes.push(minor_mode.clone());
            }
        }
        minor_modes
    }
}

#[derive(Debug, Trace)]
pub struct MinorMode {
    name: Symbol,
    data: RwLock<Value>,
    gain_focus: Procedure,
    lose_focus: Procedure,
}

impl MinorMode {
    pub fn new(
        name: Symbol,
        data: Value,
        gain_focus: Procedure,
        lose_focus: Procedure,
    ) -> Self {
        Self {
            name,
            data: RwLock::new(data),
            gain_focus,
            lose_focus,
        }
    }

    pub fn gain_focus(&self) -> Procedure {
        self.gain_focus.clone()
    }

    pub fn lose_focus(&self) -> Procedure {
        self.lose_focus.clone()
    }
}

impl SchemeCompatible for MinorMode {
    fn rtd() -> Arc<RecordTypeDescriptor>
    where
        Self: Sized
    {
        rtd!(name: "Minor mode", sealed: true)
    }
}

#[bridge(name = "minor-mode-create", lib = "(minor-mode)")]
pub fn minor_mode_create(args: &[Value]) -> Result<Vec<Value>, Exception> {
    let Some((name, rest)) = args.split_first() else {
        return Err(Exception::wrong_num_of_args(3, args.len()));
    };
    let name: Symbol = name.clone().try_into()?;
    let Some((gain_focus, rest)) = rest.split_first() else {
        return Err(Exception::wrong_num_of_args(3, args.len()));
    };
    let Some((lose_focus, rest)) = rest.split_first() else {
        return Err(Exception::wrong_num_of_args(3, args.len()));
    };
    let gain_focus: Procedure = gain_focus.clone().try_into()?;
    let lose_focus: Procedure = lose_focus.clone().try_into()?;
    let data = if let Some((data, _)) = rest.split_first() {
        data.clone()
    } else {
        Value::undefined()
    };

    let major_mode = MinorMode::new(name, data, gain_focus, lose_focus);

    Ok(vec![Value::from(Record::from_rust_type(major_mode))])
}

#[bridge(name = "minor-mode-data", lib = "(minor-mode)")]
pub async fn minor_mode_data(minor_mode: &Value) -> Result<Vec<Value>, Exception> {
    let minor_mode: Gc<MinorMode> = minor_mode.try_to_rust_type()?;
    let data = minor_mode.data.read().await.clone();

    Ok(vec![data])
}

#[bridge(name = "minor-mode-data-set!", lib = "(minor-mode)")]
pub async fn minor_mode_data_set(args: &[Value]) -> Result<Vec<Value>, Exception> {
    let Some((minor_mode, rest)) = args.split_first() else {
        return Err(Exception::wrong_num_of_args(2, args.len()));
    };
    let Some((data, _)) = rest.split_first() else {
        return Err(Exception::wrong_num_of_args(2, args.len()));
    };
    let minor_mode: Gc<MinorMode> = minor_mode.try_to_rust_type()?;
    *minor_mode.data.write().await = data.clone();

    Ok(Vec::new())
}