use crate::scheme_object::SchemeObject;

/// Represents a scheme character.
/// This type exists to provide a stronger invariance for character operations.
pub struct SchemeChar {
    base: SchemeObject
}

impl SchemeChar {
    /// Create a SchemeChar from a Rust Character
    pub fn new(c: char) -> SchemeChar {
        SchemeChar {
            base: c.into()
        }
    }
    
    /// Internal Constructor
    /// This should never be called by the user as it would violate invariance.
    pub(crate) unsafe fn from_base(base: SchemeObject) -> SchemeChar {
        SchemeChar { base }
    }
}

impl Into<SchemeObject> for SchemeChar {
    fn into(self) -> SchemeObject {
        self.base
    }
}