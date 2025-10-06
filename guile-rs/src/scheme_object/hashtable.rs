use crate::scheme_object::SchemeObject;

/// Represents a scheme hashtable.
/// This type exists to provide a stronger invariance for hashtable operations.
pub struct SchemeHashtable {
    base: SchemeObject,
}

impl SchemeHashtable {
    
    /// Construction based on size of table.
    pub fn new(size: u64) -> Self {
        let value = unsafe {
            guile_rs_sys::scm_c_make_hash_table(size)
        };
        SchemeHashtable { base: SchemeObject::from(value) }
    }
    
    /// Internal Constructor
    /// This should never be called by the user as it would violate invariance.
    pub(crate) unsafe fn from_base(base: SchemeObject) -> Self {
        SchemeHashtable { base }
    }
    
    /// Hash Ref wrapper
    /// This allows for the fetching of values from the hashtable
    pub fn get(&self, key: impl Into<SchemeObject>, default: impl Into<SchemeObject>) -> SchemeObject {
        let value = unsafe {
            guile_rs_sys::scm_hash_ref(self.base.raw, key.into().raw, default.into().raw)
        };
        SchemeObject::from(value)
    }
    
    /// Hash Set! wrapper
    /// This allows for the setting of values in the hashtable
    pub fn set(&mut self, key: impl Into<SchemeObject>, value: impl Into<SchemeObject>) {
        unsafe {
            guile_rs_sys::scm_hash_set_x(self.base.raw, key.into().raw, value.into().raw)
        };
    }
    
    /// Hash Remove! Wrapper
    /// Allows for the removal of elements in the hashtable
    pub fn remove(&mut self, key: impl Into<SchemeObject>) {
        unsafe {
            guile_rs_sys::scm_hash_remove_x(self.base.raw, key.into().raw)
        };
    }
    
    /// Hash Clear! Wrapper
    /// Allows for clearing out of all elements in the hashtable.
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