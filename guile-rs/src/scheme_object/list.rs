use crate::scheme_object::SchemeObject;

pub struct SchemeList {
    base: SchemeObject,
}

impl SchemeList {
    pub fn new(items: impl IntoIterator<Item = impl Into<SchemeObject>>) -> SchemeList {
        let vec: Vec<SchemeObject> = items.into_iter().map(Into::into).collect::<Vec<_>>();
        let mut pair = unsafe {
            guile_rs_sys::scm_eol()
        };

        for item in vec.into_iter().rev() {
            pair = unsafe {
                let list = guile_rs_sys::scm_list_1(item.raw);
                guile_rs_sys::scm_set_cdr_x(pair,  list)
            };
        }

        SchemeList { base: SchemeObject::new(pair) }
    }
    
    pub(crate) fn from_base(x: SchemeObject) -> SchemeList {
        SchemeList { base: x }
    }

    pub fn len(&self) -> usize {
        let mut len = 0;
        let mut current = self.base.raw;
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
    
    pub fn head(&self) -> SchemeObject {
        let car = unsafe {
            guile_rs_sys::rust_car(self.base.raw)
        };
        SchemeObject::from(car)
    }
    
    pub fn tail(&self) -> SchemeObject {
        let cdr = unsafe {
            guile_rs_sys::rust_cdr(self.base.raw)
        };
        SchemeObject::from(cdr)
    }
    
    pub fn append(self, other: SchemeList) -> SchemeList {
        let value = unsafe {
            let args = guile_rs_sys::scm_list_1(self.base.raw);
            guile_rs_sys::scm_set_cdr_x(args,  other.base.raw);
            guile_rs_sys::scm_append(args)
        };
        SchemeList { base: SchemeObject::from(value), }
    }
    
    pub fn reverse(self) -> SchemeList {
        let value = unsafe {
            guile_rs_sys::scm_reverse(self.base.raw)
        };
        SchemeList { base: SchemeObject::from(value), }
    }
}

impl Into<SchemeObject> for SchemeList {
    fn into(self) -> SchemeObject {
        self.base
    }
}