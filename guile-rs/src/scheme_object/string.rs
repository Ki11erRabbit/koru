use crate::scheme_object::SchemeObject;

pub struct SchemeString {
    base: SchemeObject,
}

impl SchemeString {
    pub fn new<S: AsRef<str>>(s: S) -> Self {
        let ctr = std::ffi::CString::new(s.as_ref()).unwrap();
        let value = SchemeObject::new(unsafe {
            guile_rs_sys::scm_from_utf8_string(ctr.as_ptr())
        });
        SchemeString { base: value }
    }
    
    pub(crate) fn from_base(base: SchemeObject) -> Self {
        SchemeString { base }
    }
}

impl Into<SchemeObject> for SchemeString {
    fn into(self) -> SchemeObject {
        self.base
    }
}