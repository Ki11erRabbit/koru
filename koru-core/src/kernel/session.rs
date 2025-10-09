use crate::InputSource;

pub struct Session {
    input_source: Box<dyn InputSource>,
}

impl Session {
    pub fn new(input_source: Box<dyn InputSource>) -> Self {
        Self { input_source }
    }
}