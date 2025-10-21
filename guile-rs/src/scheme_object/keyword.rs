use std::ffi::CString;
use crate::scheme_object::{SchemeObject, SchemeSymbol};

/// Represents a Scheme Keyword
/// This type provides a tighter invariance for function arguments
pub struct SchemeKeyword {
    base: SchemeObject,
}

impl SchemeKeyword {
    pub fn new<S: AsRef<str>>(name: S) -> Self {
        let name = name.as_ref();
        let name = CString::new(name).unwrap();
        
        let value = unsafe {
            guile_rs_sys::scm_from_utf8_keyword(name.as_ptr())
        };
        
        SchemeKeyword { base: SchemeObject::new(value) }
    }
    
    /// Internal Constructor
    /// This should never be called by the user as it would violate invariance.
    pub(crate) unsafe fn from_base(base: SchemeObject) -> Self {
        SchemeKeyword { base }
    }
}


impl Into<SchemeObject> for SchemeKeyword {
    fn into(self) -> SchemeObject {
        self.base
    }
}