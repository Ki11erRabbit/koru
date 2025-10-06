use crate::scheme_object::SchemeObject;

pub struct SchemeHashtable {
    base: SchemeObject,
}

impl SchemeHashtable {
    pub fn new(size: u64) -> Self {
        let value = unsafe {
            guile_rs_sys::scm_c_make_hash_table(size)
        };
        SchemeHashtable { base: SchemeObject::from(value) }
    }
    
    pub(crate) fn from_base(base: SchemeObject) -> Self {
        SchemeHashtable { base }
    }
    
    pub fn get(&self, key: impl Into<SchemeObject>, default: impl Into<SchemeObject>) -> SchemeObject {
        let value = unsafe {
            guile_rs_sys::scm_hash_ref(self.base.raw, key.into().raw, default.into().raw)
        };
        SchemeObject::from(value)
    }
    
    pub fn set(&mut self, key: impl Into<SchemeObject>, value: impl Into<SchemeObject>) {
        unsafe {
            guile_rs_sys::scm_hash_set_x(self.base.raw, key.into().raw, value.into().raw)
        };
    }
    
    pub fn remove(&mut self, key: impl Into<SchemeObject>) {
        unsafe {
            guile_rs_sys::scm_hash_remove_x(self.base.raw, key.into().raw)
        };
    }
    
    pub fn clear(&mut self) {
        unsafe {
            guile_rs_sys::scm_hash_clear_x(self.base.raw)
        };
    }
}

impl Into<SchemeObject> for SchemeHashtable {
    fn into(self) -> SchemeObject {
        self.base
    }
}