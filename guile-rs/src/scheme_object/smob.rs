use std::marker::PhantomData;
use crate::scheme_object::SchemeObject;
use crate::{SmobTag, SmobData, SmobWrapper};

/// Represents a SMOB.
/// Enforces the invariant of the SMOB's type. This allows us to access the underlying data safely
#[derive(Clone)]
pub struct SchemeSmob<T: SmobData> {
    base: SchemeObject,
    phantom: PhantomData<T>,
}

impl<T: SmobData> SchemeSmob<T> {
    
    /// Base Constructor for a SMOB.
    /// Needs a SmobTag for type safety
    pub fn new(tag: SmobTag<T>, data: T) -> Self {
        tag.make(data)
    }

    /// Internal Constructor
    /// This is incredibly unsafe compared to the other `from_base` functions.
    /// If the type is wrong then data corruption will occur.
    pub(crate) unsafe fn from_base(base: SchemeObject) -> Self {
        SchemeSmob { base, phantom: PhantomData }
    }
    
    pub fn borrow(&self) -> parking_lot::RwLockReadGuard<'_, T> {
        let ptr = unsafe {
            guile_rs_sys::rust_smob_data(*self.base.raw)
        };
        let data = unsafe { (ptr as *mut SmobWrapper<T>).as_ref() }.unwrap();
        data.borrow()
    }

    pub fn borrow_mut(&self) -> parking_lot::RwLockWriteGuard<'_, T> {
        let ptr = unsafe {
            guile_rs_sys::rust_smob_data(*self.base.raw)
        };
        let data = unsafe { (ptr as *mut SmobWrapper<T>).as_ref() }.unwrap();
        data.borrow_mut()
    }
}



impl<T: SmobData> Into<SchemeObject> for SchemeSmob<T> {
    fn into(self) -> SchemeObject {
        self.base
    }
}
