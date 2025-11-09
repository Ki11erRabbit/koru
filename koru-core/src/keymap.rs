use std::collections::HashMap;
use std::marker::PhantomData;
use scheme_rs::gc::Gc;
use crate::kernel::input::KeyPress;
use crate::kernel::scheme_api::command::Command;

enum KeyMapNode {
    Node {
        children: HashMap<KeyPress, KeyMapNode>,
    },
    Leaf {
        command: Gc<Command>,
    }
}

pub struct KeyMap {
    keys_to_command: KeyMapNode,
    default: Option<Gc<Command>>,
}

impl KeyMap {
    pub fn new_sparse() -> Self {
        Self {
            keys_to_command: KeyMapNode::Node {
                children: HashMap::new(),
            },
            default: None,
        }
    }
    
    pub fn new(default: Gc<Command>) -> Self {
        Self {
            keys_to_command: KeyMapNode::Node {
                children: HashMap::new(),
            },
            default: Some(default),
        }
    }

    pub fn lookup(&self, mut keys: &[KeyPress]) -> Option<&Gc<Command>> {
        let mut node = &self.keys_to_command;
        while keys.len() > 0 {
            match node {
                KeyMapNode::Node {
                    children,
                    ..
                } => {
                    node = match children.get(&keys[0]) {
                        Some(node) => node,
                        None => break,
                    };
                    keys = &keys[1..];
                }
                KeyMapNode::Leaf { command } => {
                    return Some(command);
                }
            }
        }
        if let Some(default) = &self.default {
            Some(default)
        } else {
            return None;
        }
    }

    pub fn add_binding(&mut self, mut keys: Vec<KeyPress>, command: Gc<Command>) {
        // Reversing keys for faster popping
        keys.reverse();
        let mut node = &mut self.keys_to_command;
        while keys.len() > 0 {
            match node {
                KeyMapNode::Node {
                    children,
                    ..
                } => {
                    if !children.contains_key(keys.last().unwrap()) {
                        children.insert(keys.last().unwrap().clone(), KeyMapNode::Node { children: HashMap::new(), });
                    }
                    node = children.get_mut(keys.last().unwrap()).unwrap();
                    keys.pop();
                }
                KeyMapNode::Leaf { .. } => {
                    let mut map = HashMap::new();
                    map.insert(keys.last().unwrap().clone(), KeyMapNode::Node { children: HashMap::new() });

                    *node = KeyMapNode::Node {
                        children: map,
                    };
                    let KeyMapNode::Node { children, .. } = node else {
                        unreachable!("we should be this value because we just set it to this value");
                    };

                    node = children.get_mut(keys.last().unwrap()).unwrap();
                    keys.pop();
                }
            }
        }
        *node = KeyMapNode::Leaf { command }
    }

    /// TODO: make it so that this can actually clear out keypresses if there is no dependencies
    pub fn remove_binding(&mut self, mut keys: &[KeyPress]) {
        let mut node = &mut self.keys_to_command;
        while keys.len() > 0 {
            match node {
                KeyMapNode::Node {
                    children,
                    ..
                } => {
                    let Some(new_node) = children.get_mut(&keys[0]) else {
                        return;
                    };
                    keys = &keys[1..];
                    node = new_node;
                }
                KeyMapNode::Leaf { .. } => {
                    *node = KeyMapNode::Node { children: HashMap::new() }
                }
            }
        }
    }
}
