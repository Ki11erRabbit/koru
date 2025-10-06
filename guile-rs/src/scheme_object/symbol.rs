use crate::scheme_object::SchemeObject;

pub struct SchemeSymbol {
    base: SchemeObject,
}

impl SchemeSymbol {
    pub fn new<S: AsRef<str>>(value: S) -> SchemeSymbol {
        let value = value.as_ref();
        let cstr = std::ffi::CString::new(value).unwrap();
        let value = unsafe {
            guile_rs_sys::scm_from_utf8_symbol(cstr.as_ptr())
        };
        
        SchemeSymbol { base: SchemeObject::from(value) }
    }
    
    pub(crate) fn from_base(base: SchemeObject) -> SchemeSymbol {
        SchemeSymbol { base }
    }
}

impl Into<SchemeObject> for SchemeSymbol {
    fn into(self) -> SchemeObject {
        self.base
    }
}