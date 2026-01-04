use std::collections::VecDeque;
use crate::logging::LogEntry;

pub struct RingBuffer {
    ring: VecDeque<LogEntry>,
    buffer_capacity: usize,
}

impl RingBuffer {
    pub fn new(buffer_capacity: usize) -> Self {
        RingBuffer {
            ring: VecDeque::with_capacity(buffer_capacity),
            buffer_capacity,
        }
    }
    
    pub fn push(&mut self, entry: LogEntry) {
        if self.ring.len() >= self.buffer_capacity {
            self.ring.pop_front();
        }
        self.ring.push_back(entry);
    }
    
    pub fn to_vec(&self) -> Vec<LogEntry> {
        self.ring.iter().cloned().collect()
    }
}