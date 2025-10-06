use std::os::raw::{c_char, c_void};
use guile_rs_sys;
pub struct Module<D> {
    name: String,
    init: Box<dyn FnOnce(&mut D) + 'static>,
    exports: Vec<String>,
}

impl<D> Module<D> {
    pub fn new<S: Into<String>>(name: S, init: Box<dyn FnOnce(&mut D) + 'static>) -> Module<D> {
        Self {
            init,
            name: name.into(),
            exports: Vec::new(),
        }
    }

    pub fn add_export<S: Into<String>>(&mut self, name: S) {
        self.exports.push(name.into());
    }

    pub fn export(&self) {
        for name in &self.exports {
            let cstr = std::ffi::CString::new(name.as_str()).unwrap();
            unsafe {
                guile_rs_sys::scm_c_export(cstr.as_ptr(), std::ptr::null_mut::<*mut c_char>());
            }
        }
    }

    pub fn define(self, data: &mut D) {
        extern "C" fn trampoline<D>(data: *mut c_void) {
            let data: Box<(Box<dyn FnOnce(&mut D) + 'static>, &mut D)> = unsafe {
                Box::from_raw(data as *mut _)
            };
            let (init, data) = *data;
            init(data);
        }
        let name = std::ffi::CString::new(self.name.as_str()).unwrap();
        let data = Box::new((self.init, data));
        let data = Box::into_raw(data);

        unsafe {
            guile_rs_sys::scm_c_define_module(name.as_ptr(), Some(trampoline::<D>), data as *mut _);
        }
    }
}