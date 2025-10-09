use crate::kernel::input::KeyPress;

pub struct KeyBuffer {
    buffer: Vec<KeyPress>
}

impl KeyBuffer {
    pub fn new() -> Self {
        KeyBuffer {
            buffer: Vec::new()
        }
    }
    
    pub fn push(&mut self, key: KeyPress) {
        self.buffer.push(key);
    }
    
    pub fn clear(&mut self) {
        self.buffer.clear();
    }
    
    pub fn is_empty(&self) -> bool {
        self.buffer.is_empty()
    }
    
    pub fn len(&self) -> usize {
        self.buffer.len()
    }
    
    pub fn get(&self) -> &[KeyPress] {
        &self.buffer
    }
}