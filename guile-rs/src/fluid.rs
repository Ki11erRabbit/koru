use crate::scheme_object::SchemeObject;
use crate::SchemeValue;

#[derive(Clone)]
pub struct FluidId(SchemeObject);

unsafe impl Send for FluidId {}
unsafe impl Sync for FluidId {}

pub struct Fluid;

impl Fluid {
    /// Creates a fluid (a scoped global variable) with a name and default value
    pub fn make_default(name: &str, value: SchemeValue) -> FluidId {
        let name = std::ffi::CString::new(name).unwrap();
        let value = unsafe {
            guile_rs_sys::scm_make_fluid_with_default(name.as_ptr(), value.into())
        };
        FluidId(SchemeObject::from(value))
    }
    
    /// Sets a fluid's value via the id
    pub fn set(id: FluidId, value: SchemeValue) {
        unsafe {
            guile_rs_sys::scm_fluid_set_x(id.0.into(), value.into())
        };
    }
    
    /// Get a fluid's value via the id
    pub fn get(id: FluidId) -> SchemeObject {
        let value = unsafe {
            guile_rs_sys::scm_fluid_ref(id.0.into())
        };
        SchemeObject::from(value)
    }
}