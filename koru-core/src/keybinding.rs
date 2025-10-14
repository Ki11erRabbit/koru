use std::collections::HashMap;
use std::marker::PhantomData;
use crate::kernel::input::KeyPress;

enum KeyBindingNode<Command: Clone> {
    Node {
        children: HashMap<KeyPress, KeyBindingNode<Command>>,
        phantom: PhantomData<Command>,
    },
    Leaf {
        command: Command,
    }
}

pub struct Keybinding<Command: Clone> {
    keys_to_command: KeyBindingNode<Command>,
}

impl<Command: Clone> Keybinding<Command> {
    pub fn new() -> Self {
        Self {
            keys_to_command: KeyBindingNode::Node {
                children: HashMap::new(),
                phantom: PhantomData
            },
        }
    }

    pub fn lookup(&self, mut keys: &[KeyPress]) -> Option<Command> {
        let mut node = &self.keys_to_command;
        while keys.len() > 0 {
            match node {
                KeyBindingNode::Node {
                    children,
                    ..
                } => {
                    node = children.get(&keys[0])?;
                    keys = &keys[1..];
                }
                KeyBindingNode::Leaf { command } => {
                    return Some(command.clone());
                }
            }
        }
        None
    }

    pub fn add_binding(&mut self, mut keys: Vec<KeyPress>, command: Command) {
        // Reversing keys for faster popping
        keys.reverse();
        let mut node = &mut self.keys_to_command;
        while keys.len() > 0 {
            match node {
                KeyBindingNode::Node {
                    children,
                    ..
                } => {
                    if !children.contains_key(keys.last().unwrap()) {
                        children.insert(keys.last().unwrap().clone(), KeyBindingNode::Node { children: HashMap::new(), phantom: PhantomData });
                    }
                    node = children.get_mut(keys.last().unwrap()).unwrap();
                    keys.pop();
                }
                KeyBindingNode::Leaf { .. } => {
                    let mut map = HashMap::new();
                    map.insert(keys.last().unwrap().clone(), KeyBindingNode::Node { children: HashMap::new(), phantom: PhantomData });

                    *node = KeyBindingNode::Node {
                        children: map,
                        phantom: PhantomData,
                    };
                    let KeyBindingNode::Node { children, .. } = node else {
                        unreachable!("we should be this value because we just set it to this value");
                    };

                    node = children.get_mut(keys.last().unwrap()).unwrap();
                    keys.pop();
                }
            }
        }
        *node = KeyBindingNode::Leaf { command }
    }
}
