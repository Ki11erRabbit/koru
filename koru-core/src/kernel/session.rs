use crate::InputSource;

pub struct Session {
    input_source: Box<dyn InputSource>,
    
}