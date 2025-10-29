mod vector;
mod list;
mod pair;
mod number;
mod procedure;
mod symbol;
mod string;
mod hashtable;
mod character;
mod smob;
mod keyword;

use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};
pub use crate::scheme_object::character::SchemeChar;
pub use crate::scheme_object::hashtable::SchemeHashtable;
pub use crate::scheme_object::list::SchemeList;
pub use crate::scheme_object::number::SchemeNumber;
pub use crate::scheme_object::pair::SchemePair;
pub use crate::scheme_object::procedure::SchemeProcedure;
pub use crate::scheme_object::string::SchemeString;
pub use crate::scheme_object::symbol::SchemeSymbol;
pub use crate::scheme_object::vector::SchemeVector;
use crate::{SchemeValue, SmobTag, SmobData};
pub use crate::scheme_object::smob::SchemeSmob;

/// Helper trait to allow for numeric types to be converted into SchemeObjects
pub trait Number: Into<SchemeObject> {}

/// Represents a generic Scheme Object that we don't know the variant
#[derive(Clone)]
pub struct SchemeObject {
    raw: Arc<guile_rs_sys::SCM>,
}

impl SchemeObject {
    /// Base Constructor
    /// Takes a raw SCM value and prevents garbage collection of it
    pub fn new(raw: guile_rs_sys::SCM) -> SchemeObject {
        unsafe {
            guile_rs_sys::scm_gc_protect_object(raw);
        }
        SchemeObject { raw: Arc::new(raw) }
    }

    pub fn protect(self) -> Self {
        unsafe {
            guile_rs_sys::scm_gc_protect_object(*self.raw);
        }
        self
    }

    /// Constructor for a Pair
    /// To get a SchemePair type use SchemePair::new instead.
    pub fn cons(x: SchemeObject, y: SchemeObject) -> SchemeObject {
        SchemePair::new(x, y).into()
    }

    /// Conditional Check for Pairs
    pub fn is_pair(&self) -> bool {
        let result = unsafe {
            guile_rs_sys::scm_pair_p(*self.raw)
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

    /// Consumes the SchemeObject and turns it into a Pair if it is one
    pub fn cast_cons(self) -> Option<SchemePair> {
        if self.is_pair() {
            Some(unsafe {
                SchemePair::from_base(self)
            })
        } else {
            None
        }
    }

    /// Constructor for a List
    /// To get a SchemeList type use SchemeList::new instead.
    pub fn list(items: impl IntoIterator<Item = impl Into<SchemeObject>>) -> SchemeObject {
       SchemeList::new(items).into()
    }

    /// Conditional check for a List value
    pub fn is_list(&self) -> bool {
        let result = unsafe {
            guile_rs_sys::scm_list_p(*self.raw)
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

    /// Consumes the SchemeObject to possibly get a SchemeList
    pub fn cast_list(self) -> Option<SchemeList> {
        if self.is_list() {
            Some(unsafe {
                SchemeList::from_base(self)
            })
        } else {
            None
        }
    }

    /// Constructor for a Vector
    /// To get a SchemeVector type use SchemeVector::new instead.
    pub fn vector(items: Vec<impl Into<SchemeObject>>) -> SchemeObject {
        SchemeVector::new(items).into()
    }

    /// Conditional check for a vector
    pub fn is_vector(&self) -> bool {
        let result = unsafe {
            guile_rs_sys::scm_is_vector(*self.raw)
        };
        if result != 0 {
            true
        } else {
            false
        }
    }

    /// Consumes the SchemeObject to possibly get a SchemeVector
    pub fn cast_vector(self) -> Option<SchemeVector> {
        if self.is_vector() {
            Some(unsafe {
                SchemeVector::from_base(self)
            })
        } else {
            None
        }
    }

    /// Constructor for a Number
    /// To get a SchemeNumber type use SchemeNumber::new instead.
    pub fn number(number: impl Number) -> SchemeObject {
        number.into()
    }

    /// Check for a number variant.
    pub fn is_number(&self) -> bool {
        let result = unsafe {
            guile_rs_sys::scm_is_number(*self.raw)
        };

        result == 1
    }

    /// Consumes the SchemeObject to possibly return a SchemeNumber
    pub fn cast_number(self) -> Option<SchemeNumber> {
        if self.is_number() {
            Some(unsafe {
                SchemeNumber::from_base(self)
            })
        } else {
            None
        }
    }

    /// Constructor for a Procedure
    /// To get a SchemeProcedure type use SchemeProcedure::new instead.
    pub fn procedure<S: AsRef<str>>(name: S) -> SchemeObject {
        SchemeProcedure::new(name.as_ref()).into()
    }

    /// Check for a procedure
    pub fn is_procedure(&self) -> bool {
        let result = unsafe {
            guile_rs_sys::scm_procedure_p(*self.raw)
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

    /// Consumes the SchemeObject to possibly return a SchemeProcedure
    pub fn cast_procedure(self) -> Option<SchemeProcedure> {
        if self.is_procedure() {
            Some(unsafe {
                SchemeProcedure::from_base(self)
            })
        } else {
            None
        }
    }

    /// Constructor for a Symbol
    /// To get a SchemeSymbol type use SchemeSymbol::new instead.
    pub fn symbol<S: AsRef<str>>(value: S) -> SchemeObject {
        SchemeSymbol::new(value).into()
    }

    /// Check for a Symbol variant
    pub fn is_symbol(&self) -> bool {
        let result = unsafe {
            guile_rs_sys::scm_symbol_p(*self.raw)
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

    /// Consumes the SchemeObject and possibly returns a SchemeSymbol
    pub fn cast_symbol(self) -> Option<SchemeSymbol> {
        if self.is_symbol() {
            Some(unsafe {
                SchemeSymbol::from_base(self)
            })
        } else {
            None
        }
    }

    /// Constructor for a String
    /// To get a SchemeString type use SchemeString::new instead.
    pub fn string<S: AsRef<str>>(value: S) -> SchemeObject {
        SchemeString::new(value).into()
    }

    /// Check for a string
    pub fn is_string(&self) -> bool {
        let result = unsafe {
            guile_rs_sys::scm_string_p(*self.raw)
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

    /// Consumes the SchemeObject and possibly returns a SchemeString
    pub fn cast_string(self) -> Option<SchemeString> {
        if self.is_string() {
            Some(unsafe {
                SchemeString::from_base(self)
            })
        } else {
            None
        }
    }

    /// Constructor for a HashTable
    /// To get a SchemeHashTable type use SchemeHashTable::new instead.
    pub fn hashtable(size: u64) -> SchemeObject {
        SchemeHashtable::new(size).into()
    }

    /// Check for a HashTable
    pub fn is_hashtable(&self) -> bool {
        let result = unsafe {
            guile_rs_sys::scm_hash_table_p(*self.raw)
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

    /// Consumes a SchemeObject to possibly return a SchemeHashtable
    pub fn cast_hashtable(self) -> Option<SchemeHashtable> {
        if self.is_hashtable() {
            Some(unsafe {
                SchemeHashtable::from_base(self)
            })
        } else {
            None
        }
    }

    /// Constructor for a Character
    /// To get a SchemeChar type use SchemeChar::new instead.
    pub fn character(c: char) -> SchemeObject {
        SchemeChar::new(c).into()
    }
    
    // Check for a character
    pub fn is_character(&self) -> bool {
        let result = unsafe {
            guile_rs_sys::scm_char_p(*self.raw)
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
    
    /// Consumes the SchemeObject and possibly returns a SchemeChar
    pub fn cast_char(self) -> Option<SchemeChar> {
        if self.is_character() {
            Some(unsafe {
                SchemeChar::from_base(self)
            })
        } else {
            None
        }
    }

    /// Constructor for a Smob
    /// To get a SchemeSmob type use SchemeSmob::new instead.
    pub fn smob<T: SmobData>(tag: SmobTag<T>, data: T) -> SchemeObject {
        SchemeSmob::new(tag, data).into()
    }
    
    /// Asserts whether or not the SchemeObject matches the Smob Type
    pub fn assert_smob<T: SmobData>(&self, tag: SmobTag<T>) {
        unsafe {
            guile_rs_sys::scm_assert_smob_type(tag.tag(), *self.raw);
        }
    }

    /// Checks if a SchemeObject is the right kind of Smob
    pub fn is_smob_type<T: SmobData>(&self, tag: SmobTag<T>) -> bool {
        let result = unsafe {
            guile_rs_sys::scm_is_smob(tag.tag(), *self.raw)
        };
        result != 0
    }
    
    /// Consumes a SchemeObject and possibly returns a SchemeSmob
    pub fn cast_smob<T: SmobData>(self, tag: SmobTag<T>) -> Option<SchemeSmob<T>> {
        if self.is_smob_type::<T>(tag) {
            Some(unsafe {
                SchemeSmob::from_base(self)
            })
        } else {
            None
        }
    }
}



impl Drop for SchemeObject {
    /// Unprotects the scheme value from garbage collection
    fn drop(&mut self) {
        if Arc::strong_count(&self.raw) == 1 {
            unsafe {
                guile_rs_sys::scm_gc_unprotect_object(*self.raw);
            }
        }
    }
}

impl From<guile_rs_sys::SCM> for SchemeObject {
    fn from(raw: guile_rs_sys::SCM) -> SchemeObject {
        SchemeObject::new(raw)
    }
}

impl From<SchemeValue> for SchemeObject {
    fn from(value: SchemeValue) -> SchemeObject {
        SchemeObject::new(value.value())
    }
}

impl Into<guile_rs_sys::SCM> for SchemeObject {
    fn into(self) -> guile_rs_sys::SCM {
        *self.raw
    }
}

impl Into<SchemeValue> for SchemeObject {
    fn into(self) -> SchemeValue {
        SchemeValue(*self.raw)
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

unsafe impl Send for SchemeObject {}
unsafe impl Sync for SchemeObject {}


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
