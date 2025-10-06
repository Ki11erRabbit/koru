mod vector;
mod list;
mod pair;
mod number;
mod procedure;
mod symbol;
mod string;
mod hashtable;
mod character;

pub use crate::scheme_object::character::SchemeChar;
pub use crate::scheme_object::hashtable::SchemeHashtable;
pub use crate::scheme_object::list::SchemeList;
pub use crate::scheme_object::number::SchemeNumber;
pub use crate::scheme_object::pair::SchemePair;
pub use crate::scheme_object::procedure::SchemeProcedure;
pub use crate::scheme_object::string::SchemeString;
pub use crate::scheme_object::symbol::SchemeSymbol;
pub use crate::scheme_object::vector::SchemeVector;

pub trait Number: Into<SchemeObject> {}

pub struct SchemeObject {
    raw: guile_rs_sys::SCM,
}

impl SchemeObject {
    fn new(raw: guile_rs_sys::SCM) -> SchemeObject {
        unsafe {
            guile_rs_sys::scm_gc_protect_object(raw);
        }
        SchemeObject { raw }
    }

    pub fn cons(x: SchemeObject, y: SchemeObject) -> SchemeObject {
        SchemePair::new(x, y).into()
    }

    pub fn is_pair(&self) -> bool {
        let result = unsafe {
            guile_rs_sys::scm_pair_p(self.raw)
        };
        let false_constant = unsafe {
            guile_rs_sys::rust_bool_false()
        };
        if result == false_constant {
            false
        } else {
            true
        }
    }

    pub fn cast_cons(self) -> Option<SchemePair> {
        if self.is_pair() {
            Some(unsafe {
                SchemePair::from_base(self)
            })
        } else {
            None
        }
    }

    pub fn list(items: impl IntoIterator<Item = impl Into<SchemeObject>>) -> SchemeObject {
       SchemeList::new(items).into()
    }

    pub fn is_list(&self) -> bool {
        let result = unsafe {
            guile_rs_sys::scm_list_p(self.raw)
        };
        let false_constant = unsafe {
            guile_rs_sys::rust_bool_false()
        };
        if result == false_constant {
            false
        } else {
            true
        }
    }

    pub fn cast_list(self) -> Option<SchemeList> {
        if self.is_list() {
            Some(unsafe {
                SchemeList::from_base(self)
            })
        } else {
            None
        }
    }

    pub fn vector(items: Vec<impl Into<SchemeObject>>) -> SchemeObject {
        SchemeVector::new(items).into()
    }

    pub fn is_vector(&self) -> bool {
        let result = unsafe {
            guile_rs_sys::scm_is_vector(self.raw)
        };
        if result != 0 {
            true
        } else {
            false
        }
    }

    pub fn cast_vector(self) -> Option<SchemeVector> {
        if self.is_vector() {
            Some(unsafe {
                SchemeVector::from_base(self)
            })
        } else {
            None
        }
    }

    pub fn number(number: impl Number) -> SchemeObject {
        number.into()
    }

    pub fn is_number(&self) -> bool {
        let result = unsafe {
            guile_rs_sys::scm_is_number(self.raw)
        };

        result == 1
    }

    pub fn cast_number(self) -> Option<SchemeNumber> {
        if self.is_number() {
            Some(unsafe {
                SchemeNumber::from_base(self)
            })
        } else {
            None
        }
    }

    pub fn procedure<S: AsRef<str>>(name: S) -> SchemeObject {
        SchemeProcedure::new(name.as_ref()).into()
    }

    pub fn is_procedure(&self) -> bool {
        let result = unsafe {
            guile_rs_sys::scm_procedure_p(self.raw)
        };
        let false_constant = unsafe {
            guile_rs_sys::rust_bool_false()
        };
        if result == false_constant {
            false
        } else {
            true
        }
    }

    pub fn cast_procedure(self) -> Option<SchemeProcedure> {
        if self.is_procedure() {
            Some(unsafe {
                SchemeProcedure::from_base(self)
            })
        } else {
            None
        }
    }

    pub fn symbol<S: AsRef<str>>(value: S) -> SchemeObject {
        SchemeSymbol::new(value).into()
    }

    pub fn is_symbol(&self) -> bool {
        let result = unsafe {
            guile_rs_sys::scm_symbol_p(self.raw)
        };
        let false_constant = unsafe {
            guile_rs_sys::rust_bool_false()
        };
        if result == false_constant {
           false
        } else {
            true
        }
    }

    pub fn cast_symbol(self) -> Option<SchemeSymbol> {
        if self.is_symbol() {
            Some(unsafe {
                SchemeSymbol::from_base(self)
            })
        } else {
            None
        }
    }

    pub fn string<S: AsRef<str>>(value: S) -> SchemeObject {
        SchemeString::new(value).into()
    }

    pub fn is_string(&self) -> bool {
        let result = unsafe {
            guile_rs_sys::scm_string_p(self.raw)
        };
        let false_constant = unsafe {
            guile_rs_sys::rust_bool_false()
        };
        if result == false_constant {
            false
        } else {
            true
        }
    }

    pub fn cast_string(self) -> Option<SchemeString> {
        if self.is_string() {
            Some(unsafe {
                SchemeString::from_base(self)
            })
        } else {
            None
        }
    }

    pub fn hashtable(size: u64) -> SchemeObject {
        SchemeHashtable::new(size).into()
    }

    pub fn is_hashtable(&self) -> bool {
        let result = unsafe {
            guile_rs_sys::scm_hash_table_p(self.raw)
        };
        let false_constant = unsafe {
            guile_rs_sys::rust_bool_false()
        };
        if result == false_constant {
            false
        } else {
            true
        }
    }

    pub fn cast_hashtable(self) -> Option<SchemeHashtable> {
        if self.is_hashtable() {
            Some(unsafe {
                SchemeHashtable::from_base(self)
            })
        } else {
            None
        }
    }
    
    pub fn character(c: char) -> SchemeObject {
        SchemeChar::new(c).into()
    }
    
    pub fn is_character(&self) -> bool {
        let result = unsafe {
            guile_rs_sys::scm_char_p(self.raw)
        };
        let false_constant = unsafe {
            guile_rs_sys::rust_bool_false()
        };
        if result == false_constant {
            false
        } else {
            true
        }
    }
    
    pub fn cast_char(self) -> Option<SchemeChar> {
        if self.is_character() {
            Some(unsafe {
                SchemeChar::from_base(self)
            })
        } else {
            None
        }
    }
}


impl Drop for SchemeObject {
    fn drop(&mut self) {
        unsafe {
            guile_rs_sys::scm_gc_unprotect_object(self.raw);
        }
    }
}

impl From<guile_rs_sys::SCM> for SchemeObject {
    fn from(raw: guile_rs_sys::SCM) -> SchemeObject {
        SchemeObject::new(raw)
    }
}

impl Into<guile_rs_sys::SCM> for SchemeObject {
    fn into(self) -> guile_rs_sys::SCM {
        self.raw
    }
}

impl From<bool> for SchemeObject {
    fn from(raw: bool) -> SchemeObject {
        if raw {
            unsafe {
                guile_rs_sys::rust_bool_true().into()
            }
        } else {
            unsafe {
                guile_rs_sys::rust_bool_false().into()
            }
        }
    }
}

impl From<char> for SchemeObject {
    fn from(c: char) -> SchemeObject {
        SchemeObject::new(unsafe {
            guile_rs_sys::scm_make_char(c as u32)
        })
    }
}

impl From<u8> for SchemeObject {
    fn from(byte: u8) -> Self {
        SchemeObject::new(unsafe {
            guile_rs_sys::scm_from_uint8(byte)
        })
    }
}

impl From<i8> for SchemeObject {
    fn from(byte: i8) -> Self {
        SchemeObject::new(unsafe {
            guile_rs_sys::scm_from_int8(byte)
        })
    }
}

impl From<u16> for SchemeObject {
    fn from(short: u16) -> Self {
        SchemeObject::new(unsafe {
            guile_rs_sys::scm_from_uint16(short)
        })
    }
}

impl From<i16> for SchemeObject {
    fn from(short: i16) -> Self {
        SchemeObject::new(unsafe {
            guile_rs_sys::scm_from_int16(short)
        })
    }
}

impl From<u32> for SchemeObject {
    fn from(int: u32) -> Self {
        SchemeObject::new(unsafe {
            guile_rs_sys::scm_from_uint32(int)
        })
    }
}


impl From<i32> for SchemeObject {
    fn from(int: i32) -> Self {
        SchemeObject::new(unsafe {
            guile_rs_sys::scm_from_int32(int)
        })
    }
}

impl From<u64> for SchemeObject {
    fn from(long: u64) -> Self {
        SchemeObject::new(unsafe {
            guile_rs_sys::scm_from_uint64(long)
        })
    }
}

impl From<i64> for SchemeObject {
    fn from(long: i64) -> Self {
        SchemeObject::new(unsafe {
            guile_rs_sys::scm_from_int64(long)
        })
    }
}

impl From<usize> for SchemeObject {
    fn from(int: usize) -> Self {
        SchemeObject::new(unsafe {
            guile_rs_sys::scm_from_uint64(int as u64)
        })
    }
}

impl From<isize> for SchemeObject {
    fn from(int: isize) -> Self {
        SchemeObject::new(unsafe {
            guile_rs_sys::scm_from_int64(int as i64)
        })
    }
}

impl From<f64> for SchemeObject {
    fn from(double: f64) -> Self {
        SchemeObject::new(unsafe {
            guile_rs_sys::scm_from_double(double)
        })
    }
}


impl Number for u8 {}
impl Number for i8 {}
impl Number for i16 {}
impl Number for u32 {}
impl Number for i32 {}
impl Number for u64 {}
impl Number for i64 {}
impl Number for f64 {}
impl Number for usize {}
impl Number for isize {}
