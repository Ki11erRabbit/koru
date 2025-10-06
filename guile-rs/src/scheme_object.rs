mod vector;
mod list;
mod pair;

use guile_rs_sys;
use crate::scheme_object::list::SchemeList;
use crate::scheme_object::pair::SchemePair;
use crate::scheme_object::vector::SchemeVector;

pub struct SchemeObject {
    raw: guile_rs_sys::SCM,
}

impl SchemeObject {
    fn new(raw: guile_rs_sys::SCM) -> SchemeObject {
        unsafe {
            guile_rs_sys::scm_gc_protect_object(raw);
        }
        SchemeObject { raw }
    }

    pub fn cons(x: SchemeObject, y: SchemeObject) -> SchemeObject {
        SchemePair::new(x, y).into()
    }
    
    pub fn is_pair(&self) -> bool {
        let result = unsafe {
            guile_rs_sys::scm_pair_p(self.raw)
        };
        let false_constant = unsafe {
            guile_rs_sys::rust_bool_false()
        };
        if result == false_constant {
            false
        } else {
            true
        }
    }

    pub fn cast_cons(self) -> Option<SchemePair> {
        if self.is_pair() {
            Some(SchemePair::from_base(self))
        } else {
            None
        }
    }

    pub fn list(items: impl IntoIterator<Item = impl Into<SchemeObject>>) -> SchemeObject {
       SchemeList::new(items).into()
    }
    
    pub fn is_list(&self) -> bool {
        let result = unsafe {
            guile_rs_sys::scm_list_p(self.raw)
        };
        let false_constant = unsafe {
            guile_rs_sys::rust_bool_false()
        };
        if result == false_constant {
            false
        } else {
            true
        }
    }
    
    pub fn cast_list(self) -> Option<SchemeList> {
        if self.is_list() {
            Some(SchemeList::from_base(self))
        } else {
            None
        }
    }

    pub fn vector(items: Vec<impl Into<SchemeObject>>) -> SchemeObject {
        SchemeVector::new(items).into()
    }
    
    pub fn is_vector(&self) -> bool {
        let result = unsafe {
            guile_rs_sys::scm_is_vector(self.raw)
        };
        if result != 0 {
            true
        } else {
            false
        }
    }
    
    pub fn cast_vector(self) -> Option<SchemeVector> {
        if self.is_vector() {
            Some(SchemeVector::from_base(self))
        } else {
            None
        }
    }
}

impl From<guile_rs_sys::SCM> for SchemeObject {
    fn from(raw: guile_rs_sys::SCM) -> SchemeObject {
        SchemeObject::new(raw)
    }
}

impl Into<guile_rs_sys::SCM> for SchemeObject {
    fn into(self) -> guile_rs_sys::SCM {
        self.raw
    }
}

impl From<bool> for SchemeObject {
    fn from(raw: bool) -> SchemeObject {
        if raw {
            unsafe {
                guile_rs_sys::rust_bool_true().into()
            }
        } else {
            unsafe {
                guile_rs_sys::rust_bool_false().into()
            }
        }
    }
}

impl From<u8> for SchemeObject {
    fn from(byte: u8) -> Self {
        SchemeObject::new(unsafe {
            guile_rs_sys::scm_from_uint8(byte)
        })
    }
}

impl From<i8> for SchemeObject {
    fn from(byte: i8) -> Self {
        SchemeObject::new(unsafe {
            guile_rs_sys::scm_from_int8(byte)
        })
    }
}

impl From<u16> for SchemeObject {
    fn from(short: u16) -> Self {
        SchemeObject::new(unsafe {
            guile_rs_sys::scm_from_uint16(short)
        })
    }
}

impl From<i16> for SchemeObject {
    fn from(short: i16) -> Self {
        SchemeObject::new(unsafe {
            guile_rs_sys::scm_from_int16(short)
        })
    }
}

impl From<u32> for SchemeObject {
    fn from(int: u32) -> Self {
        SchemeObject::new(unsafe {
            guile_rs_sys::scm_from_uint32(int)
        })
    }
}


impl From<i32> for SchemeObject {
    fn from(int: i32) -> Self {
        SchemeObject::new(unsafe {
            guile_rs_sys::scm_from_int32(int)
        })
    }
}

impl From<u64> for SchemeObject {
    fn from(long: u64) -> Self {
        SchemeObject::new(unsafe {
            guile_rs_sys::scm_from_uint64(long)
        })
    }
}

impl From<i64> for SchemeObject {
    fn from(long: i64) -> Self {
        SchemeObject::new(unsafe {
            guile_rs_sys::scm_from_int64(long)
        })
    }
}

impl From<&str> for SchemeObject {
    fn from(string: &str) -> Self {
        let ctr = std::ffi::CString::new(string).unwrap();
        SchemeObject::new(unsafe {
            guile_rs_sys::scm_from_utf8_string(ctr.as_ptr())
        })
    }
}

impl Drop for SchemeObject {
    fn drop(&mut self) {
        unsafe {
            guile_rs_sys::scm_gc_unprotect_object(self.raw);
        }
    }
}