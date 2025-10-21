use bitflags::bitflags;
use guile_rs_sys;
use crate::scheme_object::SchemeObject;

pub type SchemeValue = guile_rs_sys::SCM;

pub struct Guile;

impl Guile {
    
    /// Allows for a thread to enter into Scheme Mode
    /// This can be called in threads and even multiple times in each thread.
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

    /// Evaluates a string for a scheme expression and returns its value.
    pub fn eval(expr: &str) -> SchemeObject {
        let cstr = std::ffi::CString::new(expr).unwrap();
        unsafe {
            SchemeObject::from(guile_rs_sys::scm_c_eval_string(cstr.as_ptr()))
        }
    }

    /// Boots the system into scheme mode.
    /// This function will terminate the program if it returns.
    /// Takes in a mut ref to some data and a function that accepts that data.
    pub fn boot<D, F: FnOnce() + 'static>(main_func: F) -> ! {
        unsafe extern "C" fn trampoline<D, F: FnOnce() + 'static>(data: *mut std::os::raw::c_void, _: std::os::raw::c_int, _: *mut *mut std::os::raw::c_char) {
            let data: Box<F> = unsafe { Box::from_raw(data as *mut _) };
            let main_func = *data;
            main_func();
        }

        let args: Vec<String> = std::env::args().collect();
        let len = args.len();
        let mut argv = args.iter()
            .map(|arg| std::ffi::CString::new(arg.as_str()).unwrap())
            .map(|arg| arg.as_ptr())
            .collect::<Vec<_>>();

        let data = Box::new(main_func);

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
    
    /// Launch a scheme shell
    /// When this function returns, it will terminate the program.
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
    
    /// Defines a function to be used in scheme
    /// `name` is what the function will be called in scheme
    /// `arg_count` is the number of positional args
    /// `optional_args` is the number of optional arguments
    /// `accepts_rest` is whether or not the function is variadic
    /// `returns` the function
    pub fn define_fn(
        name: &str, 
        arg_count: i32, 
        optional_args: i32, 
        accepts_rest: bool, 
        func: extern "C" fn(SchemeValue) -> SchemeValue
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

bitflags! {
    #[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
    pub struct KeywordFlags: i32 {
        const AllowOtherKeys = guile_rs_sys::scm_allow_other_keys;
        const AllowNonkeywordArguments = guile_rs_sys::scm_allow_non_keyword_arguments;
    }
}

#[macro_export]
macro_rules! bind_keywords {
    ($func_name:expr, $rest:expr, $flags:expr, $( $keyword:expr => $out:expr ),* $(,)?) => {
        unsafe {
            $(
                let $out = $crate::scheme_object::SchemeObject::undefined();
            )*

            guile_rs_sys::scm_c_bind_keyword_arguments(
                concat!($func_name, "\0").as_ptr() as *const _,
                $rest,
                $flags,
                $(
                    guile_rs_sys::scm_from_utf8_keyword(concat!($keyword, "\0").as_ptr() as *const _),
                    &mut $out.raw,
                )*
                $crate::scheme_object::SchemeObject::undefined().raw
            );

            ( $($out.protect(),)* )
        }
    };
}