use std::panic::AssertUnwindSafe;
use std::path::Path;
use bitflags::bitflags;
use guile_rs_sys;
use crate::scheme_object::{SchemeList, SchemeObject, SchemeSymbol};
use crate::{async_module, SchemeValue};

pub struct SchemeFunction(guile_rs_sys::scm_t_subr);

pub struct Guile;

impl Guile {
    
    /// Allows for a thread to enter into Scheme Mode
    /// This can be called in threads and even multiple times in each thread.
    pub fn init<F: FnOnce() + 'static>(f: F) {
        unsafe extern "C" fn trampoline(data: *mut std::os::raw::c_void) -> *mut std::os::raw::c_void {
            async_module();
            let closure: Box<Box<dyn FnOnce()>> = unsafe {
                Box::from_raw(data as *mut _)
            };
            closure();
            std::ptr::null_mut()
        }
        let closure: Box<Box<dyn FnOnce()>> = Box::new(Box::new(f));
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
            async_module();
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
        func: impl Into<SchemeFunction>,
    ) -> SchemeObject {
        let name = std::ffi::CString::new(name).unwrap();
        let func = unsafe {
            guile_rs_sys::scm_c_define_gsubr(
                name.as_ptr(),
                arg_count,
                optional_args,
                accepts_rest.into(),
                func.into().0
            )
        };
        func.into()
    }
    
    /// Defines a value as a global accessible in Scheme
    pub fn define(
        name: &str,
        data: impl Into<SchemeObject>,
    ) -> SchemeObject {
        let name = std::ffi::CString::new(name).unwrap();
        let value = unsafe {
            guile_rs_sys::scm_c_define(
                name.as_ptr(),
                data.into().into()
            )
        };
        SchemeObject::from(value)
    }
    
    /// Finds a global variable
    pub fn lookup(name: &str) -> Option<SchemeObject> {
        let cstr = std::ffi::CString::new(name).unwrap();
        let value = unsafe {
            guile_rs_sys::scm_c_lookup(cstr.as_ptr())
        };
        if value.is_null() {
            None
        } else {
            let actual_value = unsafe {
                guile_rs_sys::scm_variable_ref(value)
            };
            Some(SchemeObject::from(actual_value))
        }
    }
    
    /// Finds a global variable that is publicly available in a module
    pub fn public_lookup(module_name: &str, name: &str) -> Option<SchemeObject> {
        let module_name = std::ffi::CString::new(module_name).unwrap();
        let name = std::ffi::CString::new(name).unwrap();
        let value = unsafe {
            guile_rs_sys::scm_c_public_lookup(module_name.as_ptr(), name.as_ptr())
        };
        if value.is_null() {
            None
        } else {
            let actual_value = unsafe {
                guile_rs_sys::scm_variable_ref(value)
            };
            Some(SchemeObject::from(actual_value))
        }
    }
    /// Finds a global variable that is private in a module
    pub fn private_lookup(module_name: &str, name: &str) -> Option<SchemeObject> {
        let module_name = std::ffi::CString::new(module_name).unwrap();
        let name = std::ffi::CString::new(name).unwrap();
        let value = unsafe {
            guile_rs_sys::scm_c_private_lookup(module_name.as_ptr(), name.as_ptr())
        };
        if value.is_null() {
            None
        } else {
            let actual_value = unsafe {
                guile_rs_sys::scm_variable_ref(value)
            };
            Some(SchemeObject::from(actual_value))
        }
    }
    
    /// Imports a module and access it to get a global
    pub fn module_lookup(module_name: &str, name: &str) -> Option<SchemeObject> {
        let module_name = std::ffi::CString::new(module_name).unwrap();
        let name = std::ffi::CString::new(name).unwrap();
        let module = unsafe {
            guile_rs_sys::scm_c_resolve_module(module_name.as_ptr())
        };
        if module.is_null() {
            return None;
        }
        let value = unsafe {
            guile_rs_sys::scm_c_module_lookup(module, name.as_ptr())
        };
        if value.is_null() {
            None
        } else {
            let actual_value = unsafe {
                guile_rs_sys::scm_variable_ref(value)
            };
            Some(SchemeObject::from(actual_value))
        }
    }
    
    /// Set a global variable
    pub fn set(name: &str, value: SchemeObject) {
        let cstr = std::ffi::CString::new(name).unwrap();
        let variable = unsafe {
            guile_rs_sys::scm_c_lookup(cstr.as_ptr())
        };
        if !variable.is_null() {
            unsafe {
                guile_rs_sys::scm_variable_set_x(variable, value.into())
            };
        }
    }

    /// Set a global variable in a module
    pub fn public_set(module_name: &str, name: &str, value: SchemeObject) {
        let module_name = std::ffi::CString::new(module_name).unwrap();
        let name = std::ffi::CString::new(name).unwrap();
        let variable = unsafe {
            guile_rs_sys::scm_c_public_lookup(module_name.as_ptr(), name.as_ptr())
        };
        if !variable.is_null() {
            unsafe {
                guile_rs_sys::scm_variable_set_x(variable, value.into())
            };
        }
    }
    /// Set a global variable in a module that is private
    pub fn private_set(module_name: &str, name: &str, value: SchemeObject) {
        let module_name = std::ffi::CString::new(module_name).unwrap();
        let name = std::ffi::CString::new(name).unwrap();
        let variable = unsafe {
            guile_rs_sys::scm_c_private_lookup(module_name.as_ptr(), name.as_ptr())
        };
        if !variable.is_null() {
            unsafe {
                guile_rs_sys::scm_variable_set_x(variable, value.into())
            };
        }
    }

    /// Set a global variable in an unresolved module.
    pub fn module_set(module_name: &str, name: &str, value: SchemeObject) {
        let module_name = std::ffi::CString::new(module_name).unwrap();
        let name = std::ffi::CString::new(name).unwrap();
        let module = unsafe {
            guile_rs_sys::scm_c_resolve_module(module_name.as_ptr())
        };
        if module.is_null() {
            return;
        }
        let variable = unsafe {
            guile_rs_sys::scm_c_module_lookup(module, name.as_ptr())
        };
        if !variable.is_null() {
            unsafe {
                guile_rs_sys::scm_variable_set_x(variable, value.into())
            };
        }
    }

    /// Raises an error for Scheme
    /// 
    /// This does not return
    pub fn throw(key: SchemeSymbol, args: SchemeList) -> ! {
        unsafe {
            guile_rs_sys::scm_throw(
                <SchemeSymbol as Into<SchemeObject>>::into(key).into(),
                <SchemeList as Into<SchemeObject>>::into(args).into())
        }
    }
    
    /// Raises a misc error for Scheme
    /// 
    /// This does not return
    pub fn misc_error(proc_name: &'static[u8], msg: &'static [u8], args: SchemeList) -> ! {
        unsafe {
            guile_rs_sys::scm_misc_error(
                proc_name.as_ptr() as *const _,
                msg.as_ptr() as *const _,
                <SchemeList as Into<SchemeObject>>::into(args).into()
            )
        }
    }

    /// Raises a type error for Scheme functions
    ///
    /// This does not return
    pub fn wrong_type_arg(proc_name: &'static [u8], pos: i32, bad_value: impl Into<SchemeObject>) -> ! {
        unsafe {
            guile_rs_sys::scm_wrong_type_arg(
                proc_name.as_ptr() as *const _,
                pos,
                bad_value.into().into()
            )
        }
    }

    /// Raises an out of range error for Scheme
    ///
    /// This does not return
    pub fn out_of_range(proc_name: &'static [u8], bad_value: impl Into<SchemeObject>) -> ! {
        unsafe {
            guile_rs_sys::scm_out_of_range(
                proc_name.as_ptr() as *const _,
                bad_value.into().into()
            )
        }
    }

    /// Raises an error for Scheme
    ///
    /// This does not return
    pub fn error(
        key: SchemeSymbol,
        proc_name: &'static [u8],
        msg: &'static [u8],
        args: SchemeList,
        rest: SchemeList
    ) -> ! {
        unsafe {
            guile_rs_sys::scm_error(
                <SchemeSymbol as Into<SchemeObject>>::into(key).into(),
                proc_name.as_ptr() as *const _,
                msg.as_ptr() as *const _,
                <SchemeList as Into<SchemeObject>>::into(args).into(),
                <SchemeList as Into<SchemeObject>>::into(rest).into(),
            )
        }
    }
    
    /// Loads and executes a scheme file specified by the path
    pub fn load<P: AsRef<Path>>(path: P) -> Result<SchemeObject, String> {
        let path = std::ffi::CString::new(path.as_ref().to_str().unwrap()).unwrap();
        
        let result = unsafe {
            std::panic::catch_unwind(AssertUnwindSafe(|| {
                guile_rs_sys::scm_c_primitive_load(path.as_ptr())
            }))
        };
        match result {
            Ok(val) => Ok(SchemeObject::new(val)),
            Err(err) => Err(format!("{:?}", err)),
        }
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
macro_rules! guile_misc_error {
    ($proc_name:literal, $msg:literal $(,)?) => {
        let list: [u8;0] = [];
        let list = $crate::scheme_object::SchemeList::new(list);
        $crate::Guile::misc_error(concat!($proc_name, "\0").as_bytes(), concat!($msg, "\0").as_bytes(), list);
    };
    ($proc_name:literal, $msg:literal, $($arg:expr),* $(,)?) => {
        let list = [ $($arg),* ];
        let list = $crate::scheme_object::SchemeList::new(list);
        $crate::Guile::misc_error(concat!($proc_name, "\0").as_bytes(), concat!($msg, "\0").as_bytes(), list);
    };
}

#[macro_export]
macro_rules! guile_wrong_type_arg {
    ($proc_name:literal, $pos:expr, $bad_value:expr) => {
        $crate::Guile::wrong_type_arg(concat!($proc_name, "\0").as_bytes(), $pos, $bad_value)
    }
}

#[macro_export]
macro_rules! guile_out_of_range {
    ($proc_name:literal, $bad_value:expr) => {
        $crate::Guile::out_of_range(concat!($proc_name, "\0").as_bytes(), $bad_value)
    }
}

#[macro_export]
macro_rules! guile_error {
    ($key:expr, $proc_name:literal, $msg:literal, $args:expr, $($rest:expr)*, $(,)?) => {
        let list = [ $($arg),* ];
        let list = $crate::scheme_object::SchemeList::new(list);
        $crate::Guile::error($key, concat!($proc_name, "\0").as_bytes(), concat!($msg, "\0").as_bytes(), $args, list)
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



impl Into<SchemeFunction> for extern "C" fn() -> SchemeValue {
    fn into(self) -> SchemeFunction {
        SchemeFunction(self as guile_rs_sys::scm_t_subr)
    }
}


impl Into<SchemeFunction> for extern "C" fn(SchemeValue) -> SchemeValue {
    fn into(self) -> SchemeFunction {
        SchemeFunction(self as guile_rs_sys::scm_t_subr)
    }
}

impl Into<SchemeFunction> for extern "C" fn(SchemeValue, SchemeValue) -> SchemeValue {
    fn into(self) -> SchemeFunction {
        SchemeFunction(self as guile_rs_sys::scm_t_subr)
    }
}

impl Into<SchemeFunction> for extern "C" fn(SchemeValue, SchemeValue, SchemeValue) -> SchemeValue {
    fn into(self) -> SchemeFunction {
        SchemeFunction(self as guile_rs_sys::scm_t_subr)
    }
}


impl Into<SchemeFunction> for extern "C" fn(SchemeValue, SchemeValue, SchemeValue, SchemeValue) -> SchemeValue {
    fn into(self) -> SchemeFunction {
        SchemeFunction(self as guile_rs_sys::scm_t_subr)
    }
}


impl Into<SchemeFunction> for extern "C" fn(SchemeValue, SchemeValue, SchemeValue, SchemeValue, SchemeValue) -> SchemeValue {
    fn into(self) -> SchemeFunction {
        SchemeFunction(self as guile_rs_sys::scm_t_subr)
    }
}

impl Into<SchemeFunction> for extern "C" fn (SchemeValue, SchemeValue, SchemeValue, SchemeValue, SchemeValue, SchemeValue) -> SchemeValue {
    fn into(self) -> SchemeFunction {
        SchemeFunction(self as guile_rs_sys::scm_t_subr)
    }
}

impl Into<SchemeFunction> for extern "C" fn (SchemeValue, SchemeValue, SchemeValue, SchemeValue, SchemeValue, SchemeValue, SchemeValue) -> SchemeValue {
    fn into(self) -> SchemeFunction {
        SchemeFunction(self as guile_rs_sys::scm_t_subr)
    }
}

impl Into<SchemeFunction> for extern "C" fn (SchemeValue, SchemeValue, SchemeValue, SchemeValue, SchemeValue, SchemeValue, SchemeValue, SchemeValue) -> SchemeValue {
    fn into(self) -> SchemeFunction {
        SchemeFunction(self as guile_rs_sys::scm_t_subr)
    }
}


impl Into<SchemeFunction> for extern "C" fn (SchemeValue, SchemeValue, SchemeValue, SchemeValue, SchemeValue, SchemeValue, SchemeValue, SchemeValue, SchemeValue) -> SchemeValue {
    fn into(self) -> SchemeFunction {
        SchemeFunction(self as guile_rs_sys::scm_t_subr)
    }
}

impl Into<SchemeFunction> for extern "C" fn (SchemeValue, SchemeValue, SchemeValue, SchemeValue, SchemeValue, SchemeValue, SchemeValue, SchemeValue, SchemeValue, SchemeValue) -> SchemeValue {
    fn into(self) -> SchemeFunction {
        SchemeFunction(self as guile_rs_sys::scm_t_subr)
    }
}

impl Into<SchemeFunction> for extern "C" fn (SchemeValue, SchemeValue, SchemeValue, SchemeValue, SchemeValue, SchemeValue, SchemeValue, SchemeValue, SchemeValue, SchemeValue, SchemeValue) -> SchemeValue {
    fn into(self) -> SchemeFunction {
        SchemeFunction(self as guile_rs_sys::scm_t_subr)
    }
}