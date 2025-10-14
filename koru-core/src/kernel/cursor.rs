
#[derive(Copy, Clone)]
pub struct Cursor {
    start: usize,
    stop: usize,
}

impl Cursor {
    pub fn new(start: usize, stop: usize) -> Self {
        Cursor { start, stop }
    }

    pub fn start(&self) -> usize {
        self.start
    }
    pub fn stop(&self) -> usize {
        self.stop
    }
}