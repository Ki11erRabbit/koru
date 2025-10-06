use crate::scheme_object::SchemeObject;

/// Represents a Scheme Vector
/// This type holds the invariance that the value is a Scheme Vector
pub struct SchemeVector {
    base: SchemeObject
}

impl SchemeVector {

    /// Constructor from a Rust Vec
    pub fn new(items: Vec<impl Into<SchemeObject>>) -> SchemeVector {
        let vector = unsafe {
            guile_rs_sys::scm_c_make_vector(items.len(), guile_rs_sys::scm_undefined())
        };
        for (i, item) in items.into_iter().map(|item| item.into()).enumerate() {
            unsafe {
                guile_rs_sys::scm_c_vector_set_x(vector, i, item.raw);
            }
        }
        SchemeVector { base: SchemeObject::new(vector) }
    }

    /// Internal Constructor
    /// This should never be called by the user as it would violate invariance.
    pub(crate) unsafe fn from_base(x: SchemeObject) -> SchemeVector {
        SchemeVector { base: x }
    }
    
    /// Fetches the length of the vector
    pub fn len(&self) -> usize {
        unsafe {
            guile_rs_sys::scm_c_vector_length(self.base.raw)
        }
    }
    
    /// Gives an iterator to the vector's contents
    pub fn iter(&self) -> VectorIter {
        VectorIter::new(self.base.raw, self.len())
    }
    
    /// Gets a value at a position in the Vector
    pub fn get(&self, index: usize) -> Option<SchemeObject> {
        if index >= self.len() {
            None
        } else {
            let result = unsafe {
                guile_rs_sys::scm_vector_ref(self.base.raw, SchemeObject::from(index).raw)
            };
            Some(SchemeObject::new(result))
        }
    }
    
    /// Sets a value at a position in the Vector
    /// This will quietly fail if the index is out of bounds
    pub fn set(&self, index: usize, value: impl Into<SchemeObject>) {
        if index >= self.len() {
            return;
        } else {
            unsafe {
                guile_rs_sys::scm_vector_set_x(self.base.raw, SchemeObject::from(index).raw, value.into().raw);
            }
        }
    }
}

impl Into<SchemeObject> for SchemeVector {
    fn into(self) -> SchemeObject {
        self.base
    }
}


pub struct VectorIter {
    vec: guile_rs_sys::SCM,
    index: usize,
    len: usize,
}

impl VectorIter {
    fn new(vec: guile_rs_sys::SCM, len: usize) -> VectorIter {
        VectorIter {
            vec,
            index: 0,
            len
        }
    }
}

impl Iterator for VectorIter {
    type Item = SchemeObject;
    fn next(&mut self) -> Option<Self::Item> {
        if self.index < self.len {
            let item = unsafe {
                guile_rs_sys::scm_c_vector_ref(self.vec, self.index)
            };
            self.index += 1;
            Some(SchemeObject::new(item))
        } else {
            None
        }
    }
}