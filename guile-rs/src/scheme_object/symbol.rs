use crate::scheme_object::SchemeObject;

/// Represents a Scheme Symbol
/// This holds the invariance that the type is a Scheme Symbol
#[derive(Clone)]
pub struct SchemeSymbol {
    base: SchemeObject,
}

impl SchemeSymbol {
    
    /// Constructor from a Rust string type
    pub fn new<S: AsRef<str>>(value: S) -> SchemeSymbol {
        let value = value.as_ref();
        let cstr = std::ffi::CString::new(value).unwrap();
        let value = unsafe {
            guile_rs_sys::scm_from_utf8_symbol(cstr.as_ptr())
        };
        
        SchemeSymbol { base: SchemeObject::from(value) }
    }

    /// Internal Constructor
    /// This should never be called by the user as it would violate invariance.
    pub(crate) unsafe fn from_base(base: SchemeObject) -> SchemeSymbol {
        SchemeSymbol { base }
    }
}

impl Into<SchemeObject> for SchemeSymbol {
    fn into(self) -> SchemeObject {
        self.base
    }
}