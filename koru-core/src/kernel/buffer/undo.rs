use std::sync::Arc;
use std::time::{Duration, SystemTime};
use tokio::sync::Mutex;


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
    pub line: usize,
    pub column: usize,
    pub value: EditValue,
}

impl EditOperation {
    pub fn new(line: usize, column: usize, value: EditValue) -> Self {
        EditOperation {
            line,
            column,
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
    line: usize,
    column: usize,
    value: UndoValue,
    children: Vec<Arc<Mutex<UndoNode>>>,
}

impl UndoNode {
    pub fn root() -> Arc<Mutex<UndoNode>> {
        let node = UndoNode {
            line: 0,
            column: 0,
            value: UndoValue::Root,
            children: Vec::new(),
        };
        Arc::new(Mutex::new(node))
    }

    pub fn new(line: usize, column: usize, value: UndoValue) -> Arc<Mutex<UndoNode>> {
        let node = Self {
            line,
            column,
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

    pub fn get_redo_branch_len(&self) -> Option<usize> {
        let Some(current) = self.current_node.clone() else {
            return None;
        };

        let guard = current.blocking_lock();
        Some(guard.children.len())
    }

    fn change_current_node(&mut self) {
        let mut node = self.root.clone();
        let mut moved_from_root = false;
        for path in &self.descent {
            moved_from_root = true;
            let child = {
                let guard = node.blocking_lock();
                guard.children[*path].clone()
            };
            node = child;
        }
        if moved_from_root {
            self.current_node = Some(node.clone());
        }
    }

    pub fn undo(&mut self) -> Option<EditOperation> {
        let Some(current) = self.current_node.clone() else {
            return None;
        };

        let edit_value = {
            let guard = current.blocking_lock();
            match &guard.value {
                UndoValue::Root => unreachable!("We should never be able to reach the root from current node"),
                UndoValue::InsertString { value, .. } => {
                    let value = EditValue::Delete {
                        count: value.chars().count(),
                    };
                    EditOperation::new(guard.line, guard.column, value)
                }
                UndoValue::DeleteString { value, .. } => {
                    let value = EditValue::Insert {
                        text: value.clone(),
                    };
                    EditOperation::new(guard.line, guard.column, value)
                }
                UndoValue::ReplaceString { old_value, new_value, .. } => {
                    let value = EditValue::Replace {
                        count: new_value.chars().count(),
                        text: old_value.clone(),
                    };
                    EditOperation::new(guard.line, guard.column, value)
                }
            }
        };

        self.descent.pop();
        self.change_current_node();
        Some(edit_value)
    }

    fn redo_internal(&mut self) -> Option<EditOperation> {
        let Some(current) = self.current_node.clone() else {
            return None;
        };

        let edit_value = {
            let guard = current.blocking_lock();
            match &guard.value {
                UndoValue::Root => unreachable!("We should never be able to reach the root from current node"),
                UndoValue::InsertString { value, .. } => {
                    let value = EditValue::Insert {
                        text: value.clone(),
                    };
                    EditOperation::new(guard.line, guard.column, value)
                }
                UndoValue::DeleteString { value, .. } => {
                    let value = EditValue::Delete {
                        count: value.chars().count(),
                    };
                    EditOperation::new(guard.line, guard.column, value)
                }
                UndoValue::ReplaceString { old_value, new_value, .. } => {
                    let value = EditValue::Replace {
                        count: old_value.chars().count(),
                        text: new_value.clone(),
                    };
                    EditOperation::new(guard.line, guard.column, value)
                }
            }
        };
        Some(edit_value)
    }

    pub fn redo_branch(&mut self, branch: usize) -> Option<EditOperation> {
        {
            let Some(current) = self.current_node.clone() else {
                return None;
            };
            {
                let guard = current.blocking_lock();
                if branch >= guard.children.len() {
                    return None;
                }
            }
            self.descent.push(branch);
            self.change_current_node();
        }

        self.redo_internal()
    }

    pub fn redo(&mut self) -> Option<EditOperation> {
        {
            let Some(current) = self.current_node.clone() else {
                return None;
            };
            let guard = current.blocking_lock();
            if guard.children.is_empty() {
                return None;
            }
            let last_branch = guard.children.len() - 1;
            self.descent.push(last_branch);
            self.change_current_node();
        }

        self.redo_internal()
    }

    pub fn insert(&mut self, line: usize, column: usize, value: String) {
        let timestamp = SystemTime::now();

        let Some(current_node) = self.current_node.clone() else {
            let value = UndoValue::InsertString {
                value,
                timestamp,
            };
            let node = UndoNode::new(line, column, value);
            self.current_node = Some(node.clone());
            let index = self.root.blocking_lock().add_child(node);
            self.descent.push(index);
            return;
        };

        let mut guard = current_node.blocking_lock();
        let guard_line = guard.line;
        let guard_column = guard.column;
        match &mut guard.value {
            UndoValue::Root => unreachable!("we should never match a root node"),
            UndoValue::InsertString { value: ins_value, timestamp: ins_timestamp} => {
                let duration = timestamp.duration_since(*ins_timestamp).unwrap();
                if duration > Duration::from_millis(100) {
                    let value = UndoValue::InsertString {
                        value,
                        timestamp,
                    };
                    let new_node = UndoNode::new(line, column, value);
                    let index = guard.add_child(new_node.clone());
                    self.descent.push(index);
                    self.current_node = Some(new_node.clone());
                    return;
                }
                if line == guard_line && column == guard_column {
                    *ins_value = value + &ins_value;
                    *ins_timestamp = timestamp;
                } else if line == guard_line && column == guard_column + ins_value.chars().count() {
                    *ins_timestamp = timestamp;
                    ins_value.push_str(&value);
                } else {
                    let value = UndoValue::InsertString {
                        value,
                        timestamp,
                    };
                    let new_node = UndoNode::new(line, column, value);
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
                let new_node = UndoNode::new(line, column, value);
                let index = guard.add_child(new_node.clone());
                self.descent.push(index);
                self.current_node = Some(new_node.clone());
            }
        }
    }

    pub fn delete(&mut self, line: usize, column: usize, value: String) {
        let timestamp = SystemTime::now();

        let Some(current_node) = self.current_node.clone() else {
            let value = UndoValue::DeleteString {
                value,
                timestamp,
            };
            let node = UndoNode::new(line, column, value);
            self.current_node = Some(node.clone());
            let index = self.root.blocking_lock().add_child(node);
            self.descent.push(index);
            return;
        };

        let mut guard = current_node.blocking_lock();
        let guard_line = guard.line;
        let guard_column = guard.column;
        match &mut guard.value {
            UndoValue::Root => unreachable!("we should never match a root node"),
            UndoValue::DeleteString { value: del_value, timestamp: del_timestamp} => {
                let duration = timestamp.duration_since(*del_timestamp).unwrap();
                if duration > Duration::from_millis(100) {
                    let value = UndoValue::DeleteString {
                        value,
                        timestamp,
                    };
                    let new_node = UndoNode::new(line, column, value);
                    let index = guard.add_child(new_node.clone());
                    self.descent.push(index);
                    self.current_node = Some(new_node.clone());
                    return;
                }
                if line == guard_line && column == guard_column {
                    *del_value = value + &del_value;
                    *del_timestamp = timestamp;
                } else if line == guard_line && column == guard_column + del_value.chars().count() {
                    *del_timestamp = timestamp;
                    del_value.push_str(&value);
                } else {
                    let value = UndoValue::DeleteString {
                        value,
                        timestamp,
                    };
                    let new_node = UndoNode::new(line, column, value);
                    let index = guard.add_child(new_node.clone());
                    self.descent.push(index);
                    self.current_node = Some(new_node.clone());
                }
            }
            _ => {
                let value = UndoValue::DeleteString {
                    value,
                    timestamp,
                };
                let new_node = UndoNode::new(line, column, value);
                let index = guard.add_child(new_node.clone());
                self.descent.push(index);
                self.current_node = Some(new_node.clone());
            }
        }
    }

    pub fn replace(&mut self, line: usize, column: usize, old_value: String, new_value: String) {
        let Some(current_node) = self.current_node.clone() else {
            let value = UndoValue::ReplaceString {
                old_value,
                new_value,
            };
            let node = UndoNode::new(line, column, value);
            self.current_node = Some(node.clone());
            let index = self.root.blocking_lock().add_child(node);
            self.descent.push(index);
            return;
        };

        let mut guard = current_node.blocking_lock();
        let value = UndoValue::ReplaceString {
            old_value,
            new_value,
        };
        let new_node = UndoNode::new(line, column, value);
        let index = guard.add_child(new_node.clone());
        self.descent.push(index);
        self.current_node = Some(new_node.clone());
    }
}