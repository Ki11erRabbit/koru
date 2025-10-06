use crate::guile::Guile;
use crate::scheme_object::SchemeObject;

pub struct SchemeProcedure {
    base: SchemeObject
}

impl SchemeProcedure {
    pub fn new(name: &str) -> SchemeProcedure {
        let proc = Guile::eval(name);
        SchemeProcedure { base: proc }
    }
    
    pub(crate) fn from_base(base: SchemeObject) -> SchemeProcedure {
        SchemeProcedure { base }
    }
    
    pub fn call(&self, args: Vec<impl Into<SchemeObject>>) -> SchemeObject {
        let mut args: Vec<guile_rs_sys::SCM> = args.into_iter().map(Into::into).map(|x| x.raw).collect();
        let len = args.len();
        let result = unsafe {
            guile_rs_sys::scm_call_n(self.base.raw, args.as_mut_ptr(), len)
        };
        SchemeObject::from(result)
    }
    
    pub fn call0(&self) -> SchemeObject {
        let result = unsafe {
            guile_rs_sys::scm_call_0(self.base.raw)
        };
        SchemeObject::new(result)
    }
    
    pub fn call1(&self, arg1: impl Into<SchemeObject>) -> SchemeObject {
        let result = unsafe {
            guile_rs_sys::scm_call_1(self.base.raw, arg1.into().raw)
        };
        SchemeObject::new(result)
    }
    
    pub fn call2(&self, arg1: impl Into<SchemeObject>, arg2: impl Into<SchemeObject>) -> SchemeObject {
        let result = unsafe {
            guile_rs_sys::scm_call_2(self.base.raw, arg1.into().raw, arg2.into().raw)
        };
        SchemeObject::new(result)
    }
    
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