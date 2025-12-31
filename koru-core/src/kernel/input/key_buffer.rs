use crate::kernel::input::KeyPress;

const KEY_BUFFER_CAPACITY: usize = 4;

pub struct KeyBuffer {
    buffer: Vec<KeyPress>
}

impl KeyBuffer {
    pub fn new() -> Self {
        KeyBuffer {
            buffer: Vec::with_capacity(KEY_BUFFER_CAPACITY)
        }
    }
    
    pub fn push(&mut self, key: KeyPress) {
        if self.buffer.len() >= KEY_BUFFER_CAPACITY {
            self.buffer.clear();
        }
        self.buffer.push(key);
        /*let string = self.buffer.iter().map(ToString::to_string).reduce(|a, b| a + " " + &b).unwrap();
        println!("{}", string);
        println!("{:?}", self.buffer);*/
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