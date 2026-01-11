use std::ops::Deref;
use std::sync::Arc;
use std::time::{Duration, SystemTime};
use tokio::sync::{Mutex, MutexGuard};

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
    },
    Bulk(Vec<EditOperation>),
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
    Transaction {
        values: Vec<UndoNode>,
        completed: bool,
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

    pub fn new_transaction(byte_offset: usize, value: UndoValue) -> UndoNode {
        Self {
            byte_offset,
            value,
            children: Vec::new(),
        }
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

    async fn undo_match(value: &UndoValue, byte_offset: usize) -> EditOperation {
        match value {
            UndoValue::Root => unreachable!("We should never be able to reach the root from current node"),
            UndoValue::InsertString { value, .. } => {
                let value = EditValue::Delete {
                    count: value.chars().count(),
                };
                EditOperation::new(byte_offset, value)
            }
            UndoValue::DeleteString { value, .. } => {
                let value = EditValue::Insert {
                    text: value.clone(),
                };
                EditOperation::new(byte_offset, value)
            }
            UndoValue::ReplaceString { old_value, new_value, .. } => {
                let value = EditValue::Replace {
                    count: new_value.chars().count(),
                    text: old_value.clone(),
                };
                EditOperation::new(byte_offset, value)
            }
            _ => panic!("We should not be able to reach a transaction from here")
        }
    }

    async fn undo_match_base(guard: impl Deref<Target = UndoNode>) -> EditOperation {
        match &guard.value {
            UndoValue::Root => unreachable!("We should never be able to reach the root from current node"),
            UndoValue::Transaction { values, .. } => {
                let mut output = Vec::new();

                for value in values.iter() {
                    let result = Self::undo_match(&value.value, value.byte_offset).await;
                    output.push(result);
                }
                EditOperation::new(0, EditValue::Bulk(output))
            }
            x => Self::undo_match(x, guard.byte_offset).await,
        }
    }

    pub async fn undo(&mut self) -> Option<EditOperation> {
        let Some(current) = self.current_node.clone() else {
            return None;
        };

        let edit_value = {
            let guard = current.lock().await;
            Self::undo_match_base(guard).await
        };

        self.descent.pop();
        self.change_current_node().await;
        Some(edit_value)
    }

    async fn redo_match(value: &UndoValue, byte_offset: usize) -> EditOperation {
        match value {
            UndoValue::Root => unreachable!("We should never be able to reach the root from current node"),
            UndoValue::InsertString { value, .. } => {
                let value = EditValue::Insert {
                    text: value.clone(),
                };
                EditOperation::new(byte_offset, value)
            }
            UndoValue::DeleteString { value, .. } => {
                let value = EditValue::Delete {
                    count: value.chars().count(),
                };
                EditOperation::new(byte_offset, value)
            }
            UndoValue::ReplaceString { old_value, new_value, .. } => {
                let value = EditValue::Replace {
                    count: old_value.chars().count(),
                    text: new_value.clone(),
                };
                EditOperation::new(byte_offset, value)
            }
            _ => panic!("We should not be able to reach a transaction from here")
        }
    }

    async fn redo_match_base(guard: impl Deref<Target = UndoNode>) -> EditOperation {
        match &guard.value {
            UndoValue::Root => unreachable!("We should never be able to reach the root from current node"),
            UndoValue::Transaction { values, .. } => {
                let mut output = Vec::new();
                for value in values.iter() {
                    let result = Self::redo_match(&value.value, value.byte_offset).await;
                    output.push(result);
                }
                EditOperation::new(0, EditValue::Bulk(output))
            }
            x => Self::redo_match(x, guard.byte_offset).await,
        }
    }

    async fn redo_internal(&mut self) -> Option<EditOperation> {
        let Some(current) = self.current_node.clone() else {
            return None;
        };

        let edit_value = {
            let guard = current.lock().await;
            Self::redo_match_base(guard).await
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

    pub async fn start_transaction(&mut self) {
        let Some(current_node) = self.current_node.clone() else {
            let value = UndoValue::Transaction {
                values: Vec::new(),
                completed: false,
            };
            let node = UndoNode::new(0, value);
            self.current_node = Some(node.clone());
            let index = self.root.lock().await.add_child(node);
            self.descent.push(index);
            return;
        };

        let mut guard = current_node.lock().await;

        let value = UndoValue::Transaction {
            values: Vec::new(),
            completed: false,
        };
        let new_node = UndoNode::new(0, value);
        let index = guard.add_child(new_node.clone());
        self.descent.push(index);
        self.current_node = Some(new_node.clone());
    }

    pub async fn end_transaction(&mut self) {
        let Some(current_node) = self.current_node.clone() else {
            panic!("A transaction should have been started before ending it.")
        };

        let mut guard = current_node.lock().await;

        match &mut guard.value {
            UndoValue::Transaction { completed, .. } => {
                *completed = true;
            }
            _ => panic!("We can only complete a transaction if the last node was a transaction")
        }
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
            UndoValue::Transaction { completed, values} => {
                if *completed {
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

                // It doesn't matter that we don't merge insert strings because
                // they will get all undone/redone at the same time anyway.
                let value = UndoValue::InsertString {
                    value,
                    timestamp,
                };
                let new_node = UndoNode::new_transaction(byte_offset, value);

                values.push(new_node);
            }
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
            UndoValue::Root => unreachable!("we should never match a root node"),
            UndoValue::Transaction { completed, values} => {
                if *completed {
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
                // It doesn't matter that we don't merge delete strings because
                // they will get all undone/redone at the same time anyway.
                let value = UndoValue::DeleteString {
                    value,
                    timestamp,
                };
                let new_node = UndoNode::new_transaction(byte_offset, value);
                values.push(new_node);

                return
            }
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
        match &mut guard.value {
            UndoValue::Root => unreachable!("we should never match a root node"),
            UndoValue::Transaction { completed, values } => {
                if *completed {
                    let value = UndoValue::ReplaceString {
                        old_value,
                        new_value,
                    };
                    let new_node = UndoNode::new(byte_offset, value);
                    let index = guard.add_child(new_node.clone());
                    self.descent.push(index);
                    self.current_node = Some(new_node.clone());
                    return;
                }

                let value = UndoValue::ReplaceString {
                    old_value,
                    new_value,
                };
                let new_node = UndoNode::new_transaction(byte_offset, value);
                values.push(new_node);
                return;
            }
            _ => {}
        }

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