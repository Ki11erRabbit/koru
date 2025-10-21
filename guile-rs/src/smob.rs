//! Definitions for SMOBs or Small Objects
//!
//! The `Smob` type provides a type id for the SMOB.
//!
//! The traits provide the interface a data type should implement to be a SMOB.

use std::{rc, sync};
use std::cell::RefCell;
use std::collections::HashMap;
use std::marker::PhantomData;
use std::rc::Rc;
use std::sync::{Arc, LazyLock, Mutex, RwLock};
use crate::scheme_object::SchemeObject;

/// A trait for defining how a Smob prints
pub trait SmobPrint {
    fn print(&self) -> String;
}

/// Defines how to drop a Smob's data when scheme's gc cleans it up.
pub trait SmobDrop {
    /// The return type is the total amount of memory freed from the heap
    fn drop(&mut self) -> usize {
        self.heap_size()
    }
    fn heap_size(&self) -> usize;
}

/// Returns the size for a SMOB.
/// This should be zero if the type is a pointer.
pub trait SmobSize: Sized {
    fn size() -> usize {
        std::mem::size_of::<Self>()
    }
}

/// Defines comparison with self and another SchemeObject
pub trait SmobEqual {
    fn eq(&self, _other: SchemeObject) -> bool {
        false
    }
}

pub trait SmobData: SmobPrint + SmobDrop + SmobSize + SmobEqual {}


/// A container for a Smob Tag
#[derive(Copy, Clone)]
pub struct Smob<T: SmobData> {
    tag: usize,
    phantom: PhantomData<T>,
}

impl<T: SmobData> Smob<T> {

    /// Registers a datatype with the scheme runtime defined by `name`.
    /// `returns` the Smob tag wrapper
    pub fn register<S: AsRef<str>>(name: S) -> Self {
        extern "C" fn smob_free<T: Sized + SmobDrop>(obj: guile_rs_sys::SCM) -> usize {
            let data = unsafe {
                guile_rs_sys::rust_smob_data(obj)
            };
            let data = data as *mut T;
            let data = unsafe { data.as_mut().unwrap() };
            data.drop()
        }
        extern "C" fn smob_print<T: Sized + SmobPrint>(obj: guile_rs_sys::SCM, port: guile_rs_sys::SCM, _pstate: *mut guile_rs_sys::scm_print_state) -> i32 {
            let data = unsafe {
                guile_rs_sys::rust_smob_data(obj)
            };
            let data = data as *mut T;
            let data = unsafe { data.as_ref().unwrap() };

            let string = data.print();
            let len = string.len();
            let cstr = std::ffi::CString::new(string).unwrap();

            unsafe {
                guile_rs_sys::scm_puts(cstr.as_ptr(), port);
            }
            len as i32
        }
        extern "C" fn smob_equal<T: Sized + SmobEqual>(obj: guile_rs_sys::SCM, other: guile_rs_sys::SCM) -> guile_rs_sys::SCM {
            let data = unsafe {
                guile_rs_sys::rust_smob_data(obj)
            };
            let data = data as *mut T;
            let data = unsafe { data.as_ref().unwrap() };

            if data.eq(other.into()) {
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
                T::size(),
            )
        };

        unsafe {
            guile_rs_sys::scm_set_smob_free(tag, Some(smob_free::<T>));
            guile_rs_sys::scm_set_smob_print(tag, Some(smob_print::<T>));
            guile_rs_sys::scm_set_smob_equalp(tag, Some(smob_equal::<T>));
        }

        Smob {
            tag,
            phantom: PhantomData,
        }
    }

    /// Creates a SMOB from the provided data
    pub fn make(&self, data: T) -> SchemeObject {
        let value = unsafe {
            guile_rs_sys::rust_new_smob(self.tag, &data as * const _ as usize)
        };
        std::mem::forget(data);
        SchemeObject::from(value)
    }

    pub fn tag(&self) -> usize {
        self.tag
    }
}

unsafe impl<T: SmobData> Send for Smob<T> {}
unsafe impl<T: SmobData> Sync for Smob<T> {}

impl<T: SmobPrint> SmobPrint for Box<T> {
    fn print(&self) -> String {
        (**self).print()
    }
}

impl<T: SmobDrop> SmobDrop for Box<T> {
    fn drop(&mut self) -> usize {
        size_of::<T>() + (**self).drop()
    }
    fn heap_size(&self) -> usize {
        size_of::<T>() + (**self).heap_size()
    }
}

impl<T> SmobSize for Box<T> {
    fn size() -> usize {
        0
    }
}

impl<T: SmobEqual> SmobEqual for Box<T> {
    fn eq(&self, other: SchemeObject) -> bool {
        (**self).eq(other)
    }
}

impl<T> SmobDrop for Vec<T> {
    fn drop(&mut self) -> usize {
        size_of::<T>() * self.capacity()
    }
    fn heap_size(&self) -> usize {
        size_of::<T>() * self.capacity()
    }
}

impl<T: SmobPrint> SmobPrint for Rc<T> {
    fn print(&self) -> String {
        (**self).print()
    }
}

impl<T: SmobDrop> SmobDrop for Rc<T> {
    fn drop(&mut self) -> usize {
        if Rc::strong_count(self) != 1 {
            0
        } else {
            (**self).heap_size()
        }
    }
    fn heap_size(&self) -> usize {
        (**self).heap_size()
    }
}

impl<T> SmobSize for Rc<T> {
    fn size() -> usize {
        size_of::<Self>()
    }
}

impl<T: SmobEqual> SmobEqual for Rc<T> {
    fn eq(&self, other: SchemeObject) -> bool {
        (**self).eq(other)
    }
}

impl<T: SmobPrint> SmobPrint for rc::Weak<T> {
    fn print(&self) -> String {
        match self.upgrade() {
            Some(obj) => obj.print(),
            None => String::new()
        }
    }
}

impl<T: SmobDrop> SmobDrop for rc::Weak<T> {
    fn drop(&mut self) -> usize {
        if rc::Weak::strong_count(self) != 1 {
            0
        } else {
            match self.upgrade() {
                Some(obj) => obj.heap_size(),
                None => 0
            }
        }
    }
    fn heap_size(&self) -> usize {
        match self.upgrade() {
            Some(obj) => obj.heap_size(),
            None => 0
        }
    }
}

impl<T> SmobSize for rc::Weak<T> {
    fn size() -> usize {
        size_of::<Self>()
    }
}

impl<T: SmobEqual> SmobEqual for rc::Weak<T> {
    fn eq(&self, other: SchemeObject) -> bool {
        match self.upgrade() {
            Some(obj) => obj.eq(other),
            None => false
        }
    }
}

impl<T: SmobPrint> SmobPrint for Arc<T> {
    fn print(&self) -> String {
        (**self).print()
    }
}

impl<T: SmobDrop> SmobDrop for Arc<T> {
    fn drop(&mut self) -> usize {
        if Arc::strong_count(self) != 1 {
            0
        } else {
            (**self).heap_size()
        }
    }
    fn heap_size(&self) -> usize {
        (**self).heap_size()
    }
}

impl<T> SmobSize for Arc<T> {
    fn size() -> usize {
        size_of::<Self>()
    }
}

impl<T: SmobEqual> SmobEqual for Arc<T> {
    fn eq(&self, other: SchemeObject) -> bool {
        (**self).eq(other)
    }
}

impl<T: SmobPrint> SmobPrint for sync::Weak<T> {
    fn print(&self) -> String {
        match self.upgrade() {
            Some(obj) => obj.print(),
            None => String::new()
        }
    }
}

impl<T: SmobDrop> SmobDrop for sync::Weak<T> {
    fn drop(&mut self) -> usize {
        if sync::Weak::strong_count(self) != 1 {
            0
        } else {
            match self.upgrade() {
                Some(obj) => obj.heap_size(),
                None => 0
            }
        }
    }
    fn heap_size(&self) -> usize {
        match self.upgrade() {
            Some(obj) => obj.heap_size(),
            None => 0
        }
    }
}

impl<T> SmobSize for sync::Weak<T> {
    fn size() -> usize {
        size_of::<Self>()
    }
}

impl<T: SmobEqual> SmobEqual for sync::Weak<T> {
    fn eq(&self, other: SchemeObject) -> bool {
        match self.upgrade() {
            Some(obj) => obj.eq(other),
            None => false
        }
    }
}


impl<T: SmobPrint> SmobPrint for Arc<Mutex<T>> {
    fn print(&self) -> String {
        self.lock().expect("Lock poisoned").print()
    }
}

impl<T: SmobDrop> SmobDrop for Arc<Mutex<T>> {
    fn drop(&mut self) -> usize {
        if Arc::strong_count(self) != 1 {
            0
        } else {
            SmobDrop::drop(&mut *self.lock().expect("Lock poisoned"))
        }
    }
    fn heap_size(&self) -> usize {
        self.lock().expect("Lock poisoned").heap_size()
    }
}

impl<T: SmobEqual> SmobEqual for Arc<Mutex<T>> {
    fn eq(&self, other: SchemeObject) -> bool {
        self.lock().expect("Lock Poisoned").eq(other)
    }
}

impl<T: SmobPrint> SmobPrint for Arc<RwLock<T>> {
    fn print(&self) -> String {
        self.read().expect("Lock Poisoned").print()
    }
}

impl<T: SmobDrop> SmobDrop for Arc<RwLock<T>> {
    fn drop(&mut self) -> usize {
        if Arc::strong_count(self) != 1 {
            0
        } else {
            SmobDrop::drop(&mut *self.write().expect("Lock poisoned"))
        }
    }
    fn heap_size(&self) -> usize {
        self.read().expect("Lock poisoned").heap_size()
    }
}


impl<T: SmobEqual> SmobEqual for Arc<RwLock<T>> {
    fn eq(&self, other: SchemeObject) -> bool {
        self.read().expect("Lock Poisoned").eq(other)
    }
}

impl<T: SmobPrint> SmobPrint for Rc<RefCell<T>> {
    fn print(&self) -> String {
        self.borrow().print()
    }
}

impl<T: SmobDrop> SmobDrop for Rc<RefCell<T>> {
    fn drop(&mut self) -> usize {
        if Rc::strong_count(self) != 1 {
            0
        } else {
            SmobDrop::drop(&mut *self.borrow_mut())
        }
    }
    fn heap_size(&self) -> usize {
        self.borrow().heap_size()
    }
}

impl<T: SmobEqual> SmobEqual for Rc<RefCell<T>> {
    fn eq(&self, other: SchemeObject) -> bool {
        self.borrow().eq(other)
    }
}