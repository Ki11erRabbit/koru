use crate::guile::Guile;
use crate::scheme_object::SchemeObject;

/// A Wrapper around a Scheme procedure
/// This type holds up the invariance that this is a procedure
pub struct SchemeProcedure {
    base: SchemeObject
}

impl SchemeProcedure {
    
    /// Constructor that fetches a procedure from a function name
    pub fn new(name: &str) -> SchemeProcedure {
        let proc = Guile::eval(name);
        SchemeProcedure { base: proc }
    }

    /// Internal Constructor
    /// This should never be called by the user as it would violate invariance.
    pub(crate) unsafe fn from_base(base: SchemeObject) -> SchemeProcedure {
        SchemeProcedure { base }
    }

    /// Invokes the function with a Rust vector for the arguments.
    /// Returns a SchemeObject value
    pub fn call(&self, args: Vec<impl Into<SchemeObject>>) -> SchemeObject {
        let mut args: Vec<guile_rs_sys::SCM> = args.into_iter().map(Into::into).map(|x| x.raw).collect();
        let len = args.len();
        let result = unsafe {
            guile_rs_sys::scm_call_n(self.base.raw, args.as_mut_ptr(), len)
        };
        SchemeObject::from(result)
    }

    /// Invokes a function with 0 arguments.
    pub fn call0(&self) -> SchemeObject {
        let result = unsafe {
            guile_rs_sys::scm_call_0(self.base.raw)
        };
        SchemeObject::new(result)
    }

    /// Invokes a function with 1 argument.
    pub fn call1(&self, arg1: impl Into<SchemeObject>) -> SchemeObject {
        let result = unsafe {
            guile_rs_sys::scm_call_1(self.base.raw, arg1.into().raw)
        };
        SchemeObject::new(result)
    }

    /// Invokes a function with 2 arguments.
    pub fn call2(&self, arg1: impl Into<SchemeObject>, arg2: impl Into<SchemeObject>) -> SchemeObject {
        let result = unsafe {
            guile_rs_sys::scm_call_2(self.base.raw, arg1.into().raw, arg2.into().raw)
        };
        SchemeObject::new(result)
    }

    /// Invokes a function with 3 arguments.
    pub fn call3(
        &self,
        arg1: impl Into<SchemeObject>,
        arg2: impl Into<SchemeObject>,
        arg3: impl Into<SchemeObject>
    ) -> SchemeObject {
        let result = unsafe {
            guile_rs_sys::scm_call_3(
                self.base.raw,
                arg1.into().raw,
                arg2.into().raw,
                arg3.into().raw
            )
        };
        SchemeObject::new(result)
    }
}

impl Into<SchemeObject> for SchemeProcedure {
    fn into(self) -> SchemeObject {
        self.base
    }
}