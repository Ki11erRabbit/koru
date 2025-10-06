use crate::scheme_object::SchemeObject;

pub trait SmobPrint {
    fn print(&self) -> String;
}

pub trait SmobDrop {
    /// The return type is the total amount of memory freed from the heap
    fn drop(&mut self) -> usize;
}

pub trait SmobSize: Sized {
    fn size() -> usize {
        std::mem::size_of::<Self>()
    }
}

pub trait SmobEqual {
    fn eq(&self, _other: SchemeObject) -> bool {
        false
    }
}

#[derive(Copy, Clone)]
pub struct Smob {
    tag: usize,
}

impl Smob {
    pub fn register<T: SmobSize + SmobPrint + SmobDrop + SmobEqual, S: AsRef<str>>(name: S) -> Self {
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
        let name = std::ffi::CString::new(name).unwrap();
        let tag = unsafe {
            guile_rs_sys::scm_make_smob_type(
                name.as_ptr(),
                T::size(),
            )
        };

        unsafe {
            guile_rs_sys::scm_set_smob_free(tag, Some(smob_free::<T>));
            guile_rs_sys::scm_set_smob_print(tag, Some(smob_print::<T>));
            guile_rs_sys::scm_set_smob_equalp(tag, Some(smob_equal::<T>));
        }

        Smob {
            tag
        }
    }
}