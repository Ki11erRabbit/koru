pub mod scheme_object;
mod guile;
mod module;
mod smob;
pub mod fluid;

pub use guile::*;
pub use module::*;
pub use smob::*;

#[derive(Clone, Copy)]
#[repr(C)]
pub struct SchemeValue(guile_rs_sys::SCM);

impl SchemeValue {
    pub fn value(&self) -> guile_rs_sys::SCM {
        self.0
    }

    pub fn undefined() -> Self {
        let value = unsafe {
            guile_rs_sys::scm_undefined()
        };
        Self(value)
    }
}