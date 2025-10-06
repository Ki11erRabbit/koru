use guile_rs_sys;
use crate::scheme_object::SchemeObject;

pub struct Guile;

impl Guile {
    pub fn init<F: FnOnce() + 'static>(f: F) {
        unsafe extern "C" fn trampoline(data: *mut std::os::raw::c_void) -> *mut std::os::raw::c_void {
            let closure: Box<Box<dyn FnOnce() -> *mut std::os::raw::c_void>> = unsafe {
                Box::from_raw(data as *mut _)
            };
            closure();
            std::ptr::null_mut()
        }
        let closure = Box::new(Box::new(f));
        unsafe {
            guile_rs_sys::scm_with_guile(
                Some(trampoline),
                Box::into_raw(closure) as *mut std::os::raw::c_void,
            );
        }
    }

    pub fn eval(expr: &str) -> SchemeObject {
        let cstr = std::ffi::CString::new(expr).unwrap();
        unsafe {
            SchemeObject::from(guile_rs_sys::scm_c_eval_string(cstr.as_ptr()))
        }
    }

    pub fn boot<D, F: FnOnce(&mut D) + 'static>(data: &mut D, main_func: F) -> ! {
        unsafe extern "C" fn trampoline<D, F: FnOnce(&mut D) + 'static>(data: *mut std::os::raw::c_void, _: std::os::raw::c_int, _: *mut *mut std::os::raw::c_char) {
            let data: Box<(&mut D, F)> = unsafe { Box::from_raw(data as *mut (&mut _, _)) };
            let (data, main_func) = *data;
            main_func(data);
        }

        let args: Vec<String> = std::env::args().collect();
        let len = args.len();
        let mut argv = args.iter()
            .map(|arg| std::ffi::CString::new(arg.as_str()).unwrap())
            .map(|arg| arg.as_ptr())
            .collect::<Vec<_>>();

        let data = Box::new((data, main_func));

        unsafe {
            guile_rs_sys::scm_boot_guile(
                len as std::os::raw::c_int,
                argv.as_mut_ptr() as *mut *mut std::os::raw::c_char,
                Some(trampoline::<D, F>),
                Box::into_raw(data) as *mut std::os::raw::c_void,
            )
        }
        // This should never be executed as scm_boot_guile calls exit
        loop {}
    }
    
    pub fn shell() -> ! {
        let args: Vec<String> = std::env::args().collect();
        let len = args.len();
        let mut argv = args.iter()
            .map(|arg| std::ffi::CString::new(arg.as_str()).unwrap())
            .map(|arg| arg.as_ptr())
            .collect::<Vec<_>>();
        
        unsafe {
            guile_rs_sys::scm_shell(len as i32, argv.as_mut_ptr() as *mut *mut std::os::raw::c_char);
        }
        // This does not return as scm_shell calls exit
        loop {}
    }
    
    pub fn define_fn(
        name: &str, 
        arg_count: i32, 
        optional_args: i32, 
        accepts_rest: bool, 
        func: extern "C" fn(guile_rs_sys::SCM) -> guile_rs_sys::SCM
    ) -> SchemeObject {
        
        let name = std::ffi::CString::new(name).unwrap();
        let func = unsafe {
            guile_rs_sys::scm_c_define_gsubr(
                name.as_ptr(),
                arg_count,
                optional_args,
                accepts_rest.into(),
                func as guile_rs_sys::scm_t_subr
            )
        };
        func.into()
    }
}