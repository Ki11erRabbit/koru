use crate::scheme_object::{SchemeList, SchemeObject};

/// Represents a string in Scheme
/// This type holds the invariance that we are a Scheme String.
#[derive(Clone)]
pub struct SchemeString {
    base: SchemeObject,
}

impl SchemeString {
    
    /// Constructor from any string type in Rust
    pub fn new<S: AsRef<str>>(s: S) -> Self {
        let ctr = std::ffi::CString::new(s.as_ref()).unwrap();
        let value = SchemeObject::new(unsafe {
            guile_rs_sys::scm_from_utf8_string(ctr.as_ptr())
        });
        SchemeString { base: value }
    }

    /// Internal Constructor
    /// This should never be called by the user as it would violate invariance.
    pub(crate) unsafe fn from_base(base: SchemeObject) -> Self {
        SchemeString { base }
    }
    
    /// Converts the String into a List
    pub fn to_list(self) -> SchemeList {
        let value = unsafe {
            guile_rs_sys::scm_string_to_list(**self.base.raw)
        };
        unsafe {
            SchemeList::from_base(SchemeObject::new(value))
        }
    }
}

impl Into<SchemeObject> for SchemeString {
    fn into(self) -> SchemeObject {
        self.base
    }
}

impl std::fmt::Display for SchemeString {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        let (c_string, c_str) = unsafe {
            let c_str = guile_rs_sys::scm_to_utf8_string(**self.base.raw);
            (std::ffi::CStr::from_ptr(c_str), c_str)
        };
        write!(f, "{}", c_string.to_str().unwrap())?;
        
        unsafe {
            libc::free(c_str as *mut _);
        }
        
        Ok(())
    }
}