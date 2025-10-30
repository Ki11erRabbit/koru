pub mod scheme_object;
mod guile;
mod module;
mod smob;
pub mod fluid;
mod asynchronous;

pub use guile::*;
pub use module::*;
pub use smob::*;
use crate::scheme_object::SchemeObject;

#[derive(Clone, Copy)]
#[repr(C)]
pub struct SchemeValue(guile_rs_sys::SCM);

impl SchemeValue {
    
    pub fn new(value: guile_rs_sys::SCM) -> Self {
        Self(value)
    }
    pub fn value(&self) -> guile_rs_sys::SCM {
        self.0
    }

    pub fn undefined() -> Self {
        let value = unsafe {
            guile_rs_sys::scm_undefined()
        };
        Self(value)
    }
}

impl From<guile_rs_sys::SCM> for SchemeValue {
    fn from(value: guile_rs_sys::SCM) -> Self {
        Self(value)
    }
}

impl From<bool> for SchemeValue {
    fn from(raw: bool) -> SchemeValue {
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

impl From<char> for SchemeValue {
    fn from(c: char) -> SchemeValue {
        Self::from(unsafe {
            guile_rs_sys::scm_make_char(c as u32)
        })
    }
}

impl From<u8> for SchemeValue {
    fn from(byte: u8) -> Self {
        SchemeValue::from(unsafe {
            guile_rs_sys::scm_from_uint8(byte)
        })
    }
}

impl From<i8> for SchemeValue {
    fn from(byte: i8) -> Self {
        SchemeValue::new(unsafe {
            guile_rs_sys::scm_from_int8(byte)
        })
    }
}

impl From<u16> for SchemeValue {
    fn from(short: u16) -> Self {
        SchemeValue::new(unsafe {
            guile_rs_sys::scm_from_uint16(short)
        })
    }
}

impl From<i16> for SchemeValue {
    fn from(short: i16) -> Self {
        SchemeValue::new(unsafe {
            guile_rs_sys::scm_from_int16(short)
        })
    }
}

impl From<u32> for SchemeValue {
    fn from(int: u32) -> Self {
        SchemeValue::new(unsafe {
            guile_rs_sys::scm_from_uint32(int)
        })
    }
}


impl From<i32> for SchemeValue {
    fn from(int: i32) -> Self {
        SchemeValue::new(unsafe {
            guile_rs_sys::scm_from_int32(int)
        })
    }
}

impl From<u64> for SchemeValue {
    fn from(long: u64) -> Self {
        SchemeValue::new(unsafe {
            guile_rs_sys::scm_from_uint64(long)
        })
    }
}

impl From<i64> for SchemeValue {
    fn from(long: i64) -> Self {
        SchemeValue::new(unsafe {
            guile_rs_sys::scm_from_int64(long)
        })
    }
}

impl From<usize> for SchemeValue {
    fn from(int: usize) -> Self {
        SchemeValue::new(unsafe {
            guile_rs_sys::scm_from_uint64(int as u64)
        })
    }
}

impl From<isize> for SchemeValue {
    fn from(int: isize) -> Self {
        SchemeValue::new(unsafe {
            guile_rs_sys::scm_from_int64(int as i64)
        })
    }
}

impl From<f64> for SchemeValue {
    fn from(double: f64) -> Self {
        SchemeValue::new(unsafe {
            guile_rs_sys::scm_from_double(double)
        })
    }
}