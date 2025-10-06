use std::marker::PhantomData;
use crate::scheme_object::SchemeObject;
use crate::{Smob, SmobData};

/// Represents a SMOB.
/// Inforces the invariant of the SMOB's type. This allows us to access the underlying data safely
pub struct SchemeSmob<T: SmobData> {
    base: SchemeObject,
    phantom: PhantomData<T>,
}

impl<T: SmobData> SchemeSmob<T> {
    
    /// Base Constructor for a SMOB.
    /// Needs a Smob for type safety
    pub fn new(tag: Smob<T>, data: T) -> Self {
        let base = tag.make(data);
        SchemeSmob { base, phantom: PhantomData }
    }

    /// Internal Constructor
    /// This is incredibly unsafe compared to the other `from_base` functions.
    /// If the type is wrong then data corruption will occur.
    pub(crate) unsafe fn from_base(base: SchemeObject) -> Self {
        SchemeSmob { base, phantom: PhantomData }
    }
}

impl<T: SmobData> std::ops::Deref for SchemeSmob<T> {
    type Target = T;
    fn deref(&self) -> &Self::Target {
        self.as_ref()
    }
}

impl<T: SmobData> std::ops::DerefMut for SchemeSmob<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.as_mut()
    }
}

impl<T: SmobData> AsRef<T> for SchemeSmob<T> {
    fn as_ref(&self) -> &T {
        let ptr = unsafe {
            guile_rs_sys::rust_smob_data(self.base.raw)
        };
        unsafe { (ptr as *mut T).as_ref() }.unwrap()
    }
}

impl<T: SmobData> AsMut<T> for SchemeSmob<T> {
    fn as_mut(&mut self) -> &mut T {
        let ptr = unsafe {
            guile_rs_sys::rust_smob_data(self.base.raw)
        };
        unsafe { (ptr as *mut T).as_mut() }.unwrap()
    }
}

impl<T: SmobData> Into<SchemeObject> for SchemeSmob<T> {
    fn into(self) -> SchemeObject {
        self.base
    }
}
