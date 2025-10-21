use std::rc::Rc;
use crate::scheme_object::SchemeObject;

/// Represents a Scheme List
/// This type provides a tighter invariance for list operations.
#[derive(Clone)]
pub struct SchemeList {
    base: SchemeObject,
}

impl SchemeList {
    /// Constructor from an iterator.
    pub fn new(items: impl IntoIterator<Item = impl Into<SchemeObject>>) -> SchemeList {
        let vec: Vec<SchemeObject> = items.into_iter().map(Into::into).collect::<Vec<_>>();
        let mut pair = unsafe {
            guile_rs_sys::scm_eol()
        };

        for item in vec.into_iter().rev() {
            pair = unsafe {
                let list = guile_rs_sys::scm_list_1(*item.raw);
                guile_rs_sys::scm_set_cdr_x(pair,  list)
            };
        }

        SchemeList { base: SchemeObject::new(pair) }
    }

    /// Internal Constructor
    /// This should never be called by the user as it would violate invariance.
    pub(crate) unsafe fn from_base(x: SchemeObject) -> SchemeList {
        SchemeList { base: x }
    }

    /// Gets the length of a list
    /// The time complexity for this operation is O(n).
    pub fn len(&self) -> usize {
        let mut len = 0;
        let mut current = *self.base.raw;
        let false_constant: guile_rs_sys::SCM = unsafe {
            guile_rs_sys::rust_bool_false()
        };

        while unsafe { guile_rs_sys::scm_null_p(current) != false_constant } {
            len += 1;
            current = unsafe {
                guile_rs_sys::rust_cdr(current)
            }
        }

        len
    }
    
    /// Gets the head of a list
    pub fn head(&self) -> SchemeObject {
        let car = unsafe {
            guile_rs_sys::rust_car(*self.base.raw)
        };
        SchemeObject::from(car)
    }
    
    /// Gets the tail of a list
    pub fn tail(&self) -> SchemeObject {
        let cdr = unsafe {
            guile_rs_sys::rust_cdr(*self.base.raw)
        };
        SchemeObject::from(cdr)
    }
    
    /// Creates a new list from two lists
    pub fn append(self, other: SchemeList) -> SchemeList {
        let value = unsafe {
            let args = guile_rs_sys::scm_list_1(*self.base.raw);
            guile_rs_sys::scm_set_cdr_x(args,  *other.base.raw);
            guile_rs_sys::scm_append(args)
        };
        SchemeList { base: SchemeObject::from(value), }
    }
    
    /// Reverses the list and returns a new one
    pub fn reverse(self) -> SchemeList {
        let value = unsafe {
            guile_rs_sys::scm_reverse(*self.base.raw)
        };
        SchemeList { base: SchemeObject::from(value), }
    }
    
    /// Creates an iterator for the list
    pub fn iter(&self) -> SchemeListIterator {
        SchemeListIterator {
            current: self.base.raw.clone()
        }
    }
}

impl Into<SchemeObject> for SchemeList {
    fn into(self) -> SchemeObject {
        self.base
    }
}

pub struct SchemeListIterator {
    current: Rc<guile_rs_sys::SCM>,
}

impl SchemeListIterator {
    fn new(list: SchemeList) -> SchemeListIterator {
        SchemeListIterator {
            current: list.base.raw.clone()
        }
    }
}

impl Iterator for SchemeListIterator {
    type Item = SchemeObject;
    fn next(&mut self) -> Option<SchemeObject> {
        let false_constant: guile_rs_sys::SCM = unsafe {
            guile_rs_sys::rust_bool_false()
        };
        if *self.current == false_constant {
            None
        } else {
            let head = unsafe {
                let head = guile_rs_sys::rust_car(*self.current);
                self.current = Rc::new(guile_rs_sys::rust_cdr(*self.current));
                head
            };
            Some(SchemeObject::from(head))
        }
    }
}