use crate::scheme_object::SchemeObject;

pub struct SchemePair {
    base: SchemeObject,
}

impl SchemePair {
    pub fn new(x: SchemeObject, y: SchemeObject) -> SchemePair {
        let pair = unsafe {
            let pair = guile_rs_sys::scm_list_1(x.raw);
            guile_rs_sys::scm_set_cdr_x(pair,  y.raw);
            pair
        };
        SchemePair { base: SchemeObject::new(pair) }
    }
    
    pub(crate) fn from_base(x: SchemeObject) -> SchemePair {
        SchemePair { base: x }
    }

    pub fn car(&self) -> SchemeObject {
        let car = unsafe {
            guile_rs_sys::rust_car(self.base.raw)
        };
        SchemeObject::from(car)
    }

    pub fn cdr(&self) -> SchemeObject {
        let cdr = unsafe {
            guile_rs_sys::rust_cdr(self.base.raw)
        };
        SchemeObject::from(cdr)
    }
}

impl Into<SchemeObject> for SchemePair {
    fn into(self) -> SchemeObject {
        self.base
    }
}