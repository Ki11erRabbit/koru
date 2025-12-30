use std::sync::Arc;
use std::time::{Duration, SystemTime};
use tokio::sync::Mutex;

static EDIT_DELAY: Duration = Duration::from_millis(1000);

pub enum EditValue {
    Delete {
        count: usize,
    },
    Insert {
        text: String,
    },
    Replace {
        count: usize,
        text: String,
    }
}

pub struct EditOperation {
    pub byte_offset: usize,
    pub value: EditValue,
}

impl EditOperation {
    pub fn new(byte_offset: usize, value: EditValue) -> Self {
        EditOperation {
            byte_offset,
            value,
        }
    }
}


enum UndoValue {
    DeleteString {
        value: String,
        timestamp: SystemTime,
    },
    InsertString {
        value: String,
        timestamp: SystemTime,
    },
    ReplaceString {
        old_value: String,
        new_value: String,
    },
    Root
}

struct UndoNode {
    byte_offset: usize,
    value: UndoValue,
    children: Vec<Arc<Mutex<UndoNode>>>,
}

impl UndoNode {
    pub fn root() -> Arc<Mutex<UndoNode>> {
        let node = UndoNode {
            byte_offset: 0,
            value: UndoValue::Root,
            children: Vec::new(),
        };
        Arc::new(Mutex::new(node))
    }

    pub fn new(byte_offset: usize, value: UndoValue) -> Arc<Mutex<UndoNode>> {
        let node = Self {
            byte_offset,
            value,
            children: Vec::new(),
        };
        Arc::new(Mutex::new(node))
    }

    pub fn add_child(&mut self, child: Arc<Mutex<UndoNode>>) -> usize {
        let index = self.children.len();
        self.children.push(child);
        index
    }
}

pub struct UndoTree {
    root: Arc<Mutex<UndoNode>>,
    current_node: Option<Arc<Mutex<UndoNode>>>,
    descent: Vec<usize>,
}

impl UndoTree {
    pub fn new() -> Self {
        let root = UndoNode::root();
        UndoTree {
            current_node: None,
            root,
            descent: Vec::new(),
        }
    }

    pub async fn get_redo_branch_len(&self) -> Option<usize> {
        let Some(current) = self.current_node.clone() else {
            return None;
        };

        let guard = current.lock().await;
        Some(guard.children.len())
    }

    async fn change_current_node(&mut self) {
        let mut node = self.root.clone();
        let mut moved_from_root = false;
        for path in &self.descent {
            moved_from_root = true;
            let child = {
                let guard = node.lock().await;
                guard.children[*path].clone()
            };
            node = child;
        }
        if moved_from_root {
            self.current_node = Some(node.clone());
        } else if self.descent.is_empty() {
            self.current_node = None;
        }
    }

    pub async fn undo(&mut self) -> Option<EditOperation> {
        let Some(current) = self.current_node.clone() else {
            return None;
        };

        let edit_value = {
            let guard = current.lock().await;
            match &guard.value {
                UndoValue::Root => unreachable!("We should never be able to reach the root from current node"),
                UndoValue::InsertString { value, .. } => {
                    let value = EditValue::Delete {
                        count: value.chars().count(),
                    };
                    EditOperation::new(guard.byte_offset, value)
                }
                UndoValue::DeleteString { value, .. } => {
                    let value = EditValue::Insert {
                        text: value.clone(),
                    };
                    EditOperation::new(guard.byte_offset, value)
                }
                UndoValue::ReplaceString { old_value, new_value, .. } => {
                    let value = EditValue::Replace {
                        count: new_value.chars().count(),
                        text: old_value.clone(),
                    };
                    EditOperation::new(guard.byte_offset, value)
                }
            }
        };

        self.descent.pop();
        self.change_current_node().await;
        Some(edit_value)
    }

    async fn redo_internal(&mut self) -> Option<EditOperation> {
        let Some(current) = self.current_node.clone() else {
            return None;
        };

        let edit_value = {
            let guard = current.lock().await;
            match &guard.value {
                UndoValue::Root => unreachable!("We should never be able to reach the root from current node"),
                UndoValue::InsertString { value, .. } => {
                    let value = EditValue::Insert {
                        text: value.clone(),
                    };
                    EditOperation::new(guard.byte_offset, value)
                }
                UndoValue::DeleteString { value, .. } => {
                    let value = EditValue::Delete {
                        count: value.chars().count(),
                    };
                    EditOperation::new(guard.byte_offset, value)
                }
                UndoValue::ReplaceString { old_value, new_value, .. } => {
                    let value = EditValue::Replace {
                        count: old_value.chars().count(),
                        text: new_value.clone(),
                    };
                    EditOperation::new(guard.byte_offset, value)
                }
            }
        };
        Some(edit_value)
    }

    pub async fn redo_branch(&mut self, branch: usize) -> Option<EditOperation> {
        {
            let Some(current) = self.current_node.clone() else {
                return None;
            };
            {
                let guard = current.lock().await;
                if branch >= guard.children.len() {
                    return None;
                }
            }
            self.descent.push(branch);
            self.change_current_node().await;
        }

        self.redo_internal().await
    }

    pub async fn redo(&mut self) -> Option<EditOperation> {
        {
            let current = match self.current_node.clone() {
                Some(current) => current,
                None => self.root.clone(),
            };
            let last_branch = {
                let guard = current.lock().await;
                if guard.children.is_empty() {
                    return None;
                }
                guard.children.len() - 1
            };
            self.descent.push(last_branch);
            self.change_current_node().await;
        }

        self.redo_internal().await
    }

    pub async fn insert(&mut self, byte_offset: usize, value: String) {
        let timestamp = SystemTime::now();

        let Some(current_node) = self.current_node.clone() else {
            let value = UndoValue::InsertString {
                value,
                timestamp,
            };
            let node = UndoNode::new(byte_offset, value);
            self.current_node = Some(node.clone());
            let index = self.root.lock().await.add_child(node);
            self.descent.push(index);
            return;
        };

        let mut guard = current_node.lock().await;
        let guard_byte_offset = guard.byte_offset;
        match &mut guard.value {
            UndoValue::Root => unreachable!("we should never match a root node"),
            UndoValue::InsertString { value: ins_value, timestamp: ins_timestamp} => {
                let duration = timestamp.duration_since(*ins_timestamp).unwrap();
                if duration > EDIT_DELAY {
                    let value = UndoValue::InsertString {
                        value,
                        timestamp,
                    };
                    let new_node = UndoNode::new(byte_offset, value);
                    let index = guard.add_child(new_node.clone());
                    self.descent.push(index);
                    self.current_node = Some(new_node.clone());
                    return;
                }
                if byte_offset == guard_byte_offset {
                    *ins_value = value + &ins_value;
                    *ins_timestamp = timestamp;
                } else if byte_offset == guard_byte_offset + ins_value.len() {
                    *ins_timestamp = timestamp;
                    ins_value.push_str(&value);
                } else {
                    let value = UndoValue::InsertString {
                        value,
                        timestamp,
                    };
                    let new_node = UndoNode::new(byte_offset, value);
                    let index = guard.add_child(new_node.clone());
                    self.descent.push(index);
                    self.current_node = Some(new_node.clone());
                }
            }
            _ => {
                let value = UndoValue::InsertString {
                    value,
                    timestamp,
                };
                let new_node = UndoNode::new(byte_offset, value);
                let index = guard.add_child(new_node.clone());
                self.descent.push(index);
                self.current_node = Some(new_node.clone());
            }
        }
    }

    pub async fn delete(&mut self, byte_offset: usize, value: String) {
        let timestamp = SystemTime::now();

        let Some(current_node) = self.current_node.clone() else {
            let value = UndoValue::DeleteString {
                value,
                timestamp,
            };
            let node = UndoNode::new(byte_offset, value);
            self.current_node = Some(node.clone());
            let index = self.root.lock().await.add_child(node);
            self.descent.push(index);
            return;
        };

        let mut guard = current_node.lock().await;
        let guard_byte_offset = guard.byte_offset;
        let new_offset = match &mut guard.value {
            UndoValue::DeleteString { value: del_value, timestamp: del_timestamp} => {
                let duration = timestamp.duration_since(*del_timestamp).unwrap();
                if duration > EDIT_DELAY {
                    let value = UndoValue::DeleteString {
                        value,
                        timestamp,
                    };
                    let new_node = UndoNode::new(byte_offset, value);
                    let index = guard.add_child(new_node.clone());
                    self.descent.push(index);
                    self.current_node = Some(new_node.clone());
                    return;
                }
                if byte_offset == guard_byte_offset {
                    del_value.push_str(&value);
                    *del_timestamp = timestamp;
                    guard_byte_offset
                } else if byte_offset == guard_byte_offset - value.len() {
                    *del_value = value + &del_value;
                    *del_timestamp = timestamp;
                    byte_offset
                } else {
                    let value = UndoValue::DeleteString {
                        value,
                        timestamp,
                    };
                    let new_node = UndoNode::new(byte_offset, value);
                    let index = guard.add_child(new_node.clone());
                    self.descent.push(index);
                    self.current_node = Some(new_node.clone());
                    return;
                }
            }
            _ => {
                let value = UndoValue::DeleteString {
                    value,
                    timestamp,
                };
                let new_node = UndoNode::new(byte_offset, value);
                let index = guard.add_child(new_node.clone());
                self.descent.push(index);
                self.current_node = Some(new_node.clone());
                return;
            }
        };
        guard.byte_offset = new_offset;
    }

    pub async fn replace(&mut self, byte_offset: usize, old_value: String, new_value: String) {
        let Some(current_node) = self.current_node.clone() else {
            let value = UndoValue::ReplaceString {
                old_value,
                new_value,
            };
            let node = UndoNode::new(byte_offset, value);
            self.current_node = Some(node.clone());
            let index = self.root.lock().await.add_child(node);
            self.descent.push(index);
            return;
        };

        let mut guard = current_node.lock().await;
        let value = UndoValue::ReplaceString {
            old_value,
            new_value,
        };
        let new_node = UndoNode::new(byte_offset, value);
        let index = guard.add_child(new_node.clone());
        self.descent.push(index);
        self.current_node = Some(new_node.clone());
    }
}