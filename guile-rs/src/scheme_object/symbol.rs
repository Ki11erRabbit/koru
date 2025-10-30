use std::ffi::CStr;
use guile_rs_sys::scm_to_utf8_string;
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

impl std::fmt::Display for SchemeSymbol {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        let string = unsafe {
            guile_rs_sys::scm_symbol_to_string(**self.base.raw)
        };
        let ptr = unsafe {
            scm_to_utf8_string(string)
        };
        let cstr = unsafe {
            CStr::from_ptr(ptr as *const i8)
        };
        let result = write!(f, "{}", cstr.to_str().expect("we were not valid UTF-8"));
        unsafe {
            libc::free(ptr as *mut _);
        }
        result
    }
}