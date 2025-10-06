use crate::scheme_object::SchemeObject;

pub struct SchemeVector {
    base: SchemeObject
}

impl SchemeVector {

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
    
    pub(crate) fn from_base(x: SchemeObject) -> SchemeVector {
        SchemeVector { base: x }
    }
    
    pub fn len(&self) -> usize {
        unsafe {
            guile_rs_sys::scm_c_vector_length(self.base.raw)
        }
    }
    
    pub fn iter(&self) -> VectorIter {
        VectorIter::new(self.base.raw, self.len())
    }
    
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