use crate::scheme_object::SchemeObject;

/// Represents a simple pair in Scheme or improper list
/// This type holds up the invariance that the scheme value is a pair.
pub struct SchemePair {
    base: SchemeObject,
}

impl SchemePair {
    
    /// Constructor that takes 2 elements.
    pub fn new(x: SchemeObject, y: SchemeObject) -> SchemePair {
        let pair = unsafe {
            let pair = guile_rs_sys::scm_list_1(x.raw);
            guile_rs_sys::scm_set_cdr_x(pair,  y.raw);
            pair
        };
        SchemePair { base: SchemeObject::new(pair) }
    }

    /// Internal Constructor
    /// This should never be called by the user as it would violate invariance.
    pub(crate) unsafe fn from_base(x: SchemeObject) -> SchemePair {
        SchemePair { base: x }
    }
    
    /// Constructor that takes in a Rust tuple.
    pub fn from_tuple((x, y): (impl Into<SchemeObject>, impl Into<SchemeObject>)) -> SchemePair {
        SchemePair::new(x.into(), y.into())
    }

    /// Fetches the CAR value
    pub fn car(&self) -> SchemeObject {
        let car = unsafe {
            guile_rs_sys::rust_car(self.base.raw)
        };
        SchemeObject::from(car)
    }

    /// Fetches the CDR value
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