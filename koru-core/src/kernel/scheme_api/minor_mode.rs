use std::collections::{HashMap, VecDeque};
use scheme_rs::gc::Gc;
use scheme_rs::value::Value;
use tokio::sync::RwLock;

#[derive(Clone)]
pub struct MinorModeManager {
    minor_modes: Vec<Option<Gc<MinorMode>>>,
    map: HashMap<String, usize>,
    free_list: VecDeque<usize>,
}

impl MinorModeManager {
    pub fn new() -> Self {
        Self {
            minor_modes: Vec::new(),
            map: HashMap::new(),
            free_list: VecDeque::new(),
        }
    }

    pub fn add_minor_mode(&mut self, minor_mode: Gc<MinorMode>) {
        if self.map.contains_key(&minor_mode.name) {
            return;
        }
        if let Some(index) = self.free_list.pop_front() {
            self.minor_modes[index] = Some(minor_mode);
        } else {
            let index = self.free_list.len();
            let name = minor_mode.name.clone();
            self.map.insert(name, index);
            self.minor_modes.push(Some(minor_mode));
        }
    }

    pub fn remove_minor_mode(&mut self, minor_mode_name: &str) -> Option<String> {
        if let Some(index) = self.map.remove(minor_mode_name) {
            self.minor_modes[index] = None;
            self.free_list.push_back(index);
            Some(minor_mode_name.to_string())
        } else {
            None
        }
    }

    pub fn get_minor_mode(&self, minor_mode_name: &str) -> Option<&Gc<MinorMode>> {
        if let Some(index) = self.map.get(minor_mode_name) {
            self.minor_modes[*index].as_ref()
        } else {
            None
        }
    }
}

pub struct MinorMode {
    name: String,
    data: RwLock<Value>,
}