use guile_rs_sys;

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
}

impl From<i8> for SchemeObject {
    fn from(byte: i8) -> Self {
        SchemeObject::new(unsafe {
            guile_rs_sys::scm_from_int8(byte)
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

impl From<i32> for SchemeObject {
    fn from(int: i32) -> Self {
        SchemeObject::new(unsafe {
            guile_rs_sys::scm_from_int32(int)
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

impl Drop for SchemeObject {
    fn drop(&mut self) {
        unsafe {
            guile_rs_sys::scm_gc_unprotect_object(self.raw);
        }
    }
}