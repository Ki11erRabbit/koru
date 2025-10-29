//! Definitions for SMOBs or Small Objects
//!
//! The `Smob` type provides a type id for the SMOB.
//!
//! The traits provide the interface a data type should implement to be a SMOB.


use std::any::type_name;
use std::marker::PhantomData;
use crate::scheme_object::{SchemeObject, SchemeSmob};


pub enum SmobWrapper<T: SmobData> {
    Blank,
    Data(parking_lot::RwLock<T>),
}

impl<T: SmobData> SmobWrapper<T> {

    pub fn borrow(&self) -> parking_lot::RwLockReadGuard<T> {
        match self {
            SmobWrapper::Blank => unreachable!("Value already freed"),
            SmobWrapper::Data(r) => r.read(),
        }
    }

    pub fn borrow_mut(&self) -> parking_lot::RwLockWriteGuard<T> {
        match self {
            SmobWrapper::Blank => unreachable!("Value already freed"),
            SmobWrapper::Data(r) => r.write(),
        }
    }

    fn print(&self) -> String {
        self.borrow().print()
    }

    fn heap_size(&self) -> usize {
        self.borrow().heap_size()
    }

    fn eq(&self, other: SchemeObject) -> bool {
        self.borrow().eq(other)
    }
}


pub trait SmobData: Sized {

    /// Provides a way for a Smob to display itself
    ///
    /// It should be in the format of `#<TYPENAME>`
    fn print(&self) -> String {
        format!("#<{}>", type_name::<Self>())
    }
    /// The return type is the total amount of memory to be freed from the heap
    fn heap_size(&self) -> usize;
    /// Defines comparison with self and another SchemeObject
    fn eq(&self, _other: SchemeObject) -> bool {
        false
    }
    fn size() -> usize {
        std::mem::size_of::<Self>()
    }
}


/// A container for a Smob Tag
#[derive(Copy, Clone)]
pub struct SmobTag<T: SmobData> {
    tag: usize,
    phantom: PhantomData<T>,
}

impl<T: SmobData> SmobTag<T> {

    /// Registers a datatype with the scheme runtime defined by `name`.
    /// `returns` the Smob tag wrapper
    pub fn register<S: AsRef<str>>(name: S) -> Self {
        extern "C" fn smob_free<T: SmobData>(obj: guile_rs_sys::SCM) -> usize {
            let data = unsafe {
                guile_rs_sys::rust_smob_data(obj)
            };
            let data = data as *mut SmobWrapper<T>;
            let data = unsafe { data.as_mut().unwrap() };
            let data = std::mem::replace(data, SmobWrapper::Blank);

            data.heap_size()
        }
        extern "C" fn smob_print<T: SmobData>(obj: guile_rs_sys::SCM, port: guile_rs_sys::SCM, _pstate: *mut guile_rs_sys::scm_print_state) -> i32 {
            let data = unsafe {
                guile_rs_sys::rust_smob_data(obj)
            };
            let data = data as *mut SmobWrapper<T>;
            let data = unsafe { data.as_ref().unwrap() };

            let string = data.print();
            let len = string.len();
            let cstr = std::ffi::CString::new(string).unwrap();

            unsafe {
                guile_rs_sys::scm_puts(cstr.as_ptr(), port);
            }
            len as i32
        }
        extern "C" fn smob_equal<T: SmobData>(obj: guile_rs_sys::SCM, other: guile_rs_sys::SCM) -> guile_rs_sys::SCM {
            let data = unsafe {
                guile_rs_sys::rust_smob_data(obj)
            };
            let data = data as *mut SmobWrapper<T>;
            let data = unsafe { data.as_ref().unwrap() };

            if data.eq(SchemeObject::new(other)) {
                unsafe {
                    guile_rs_sys::rust_bool_true()
                }
            } else {
                unsafe {
                    guile_rs_sys::rust_bool_false()
                }
            }
        }

        let name = name.as_ref();
        let cstr = std::ffi::CString::new(name).unwrap();
        let tag = unsafe {
            guile_rs_sys::scm_make_smob_type(
                cstr.as_ptr(),
                size_of::<SmobWrapper<T>>(),
            )
        };

        unsafe {
            guile_rs_sys::scm_set_smob_free(tag, Some(smob_free::<T>));
            guile_rs_sys::scm_set_smob_print(tag, Some(smob_print::<T>));
            guile_rs_sys::scm_set_smob_equalp(tag, Some(smob_equal::<T>));
        }

        SmobTag {
            tag,
            phantom: PhantomData,
        }
    }

    /// Creates a SMOB from the provided data
    pub fn make(&self, data: T) -> SchemeSmob<T> {
        let data = SmobWrapper::Data(parking_lot::RwLock::new(data));

        let value = unsafe {
            guile_rs_sys::rust_new_smob(self.tag, &data as * const _ as usize)
        };
        std::mem::forget(data);
        unsafe {
            SchemeSmob::from_base(SchemeObject::new(value))
        }
    }

    pub fn tag(&self) -> usize {
        self.tag
    }
}

unsafe impl<T: SmobData> Send for SmobTag<T> {}
unsafe impl<T: SmobData> Sync for SmobTag<T> {}

impl<T: SmobData> SmobData for Box<T> {
    fn print(&self) -> String {
        (**self).print()
    }
    fn heap_size(&self) -> usize {
        size_of::<T>() + (**self).heap_size()
    }
    fn eq(&self, other: SchemeObject) -> bool {
        (**self).eq(other)
    }

    fn size() -> usize {
        0
    }
}