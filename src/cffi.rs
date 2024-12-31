use std::collections::HashMap;
use std::ffi::{c_void, CStr, CString};
use std::os::raw::{c_char, c_double, c_int};
use std::ptr;
use std::rc::Rc;
use std::slice;

use crate::runtime::Runtime;
use crate::value::{Number, Value};

#[repr(C)]
pub struct CRuntime(*mut Runtime);

#[no_mangle]
pub extern "C" fn runtime_new() -> CRuntime {
    CRuntime(Box::into_raw(Box::new(Runtime::new())))
}

#[no_mangle]
pub extern "C" fn runtime_from(path: *const c_char) -> CRuntime {
    let c_str = unsafe { CStr::from_ptr(path) };
    let path_str = c_str.to_str().unwrap();
    let path_buf = std::path::PathBuf::from(path_str);

    match Runtime::from_file(path_buf) {
        Ok(runtime) => CRuntime(Box::into_raw(Box::new(runtime))),
        Err(_) => CRuntime(ptr::null_mut()),
    }
}

#[no_mangle]
pub extern "C" fn runtime_load_file(runtime: CRuntime, path: *const c_char) -> c_int {
    let runtime = unsafe { runtime.0.as_mut() }.unwrap();
    let c_str = unsafe { CStr::from_ptr(path) };
    let path_str = c_str.to_str().unwrap();
    let path_buf = std::path::PathBuf::from(path_str);

    match runtime.load_file(path_buf) {
        Ok(_) => 0,
        Err(_) => -1,
    }
}

#[no_mangle]
pub extern "C" fn runtime_load_vtc(runtime: CRuntime, input: *const c_char) -> c_int {
    let runtime = unsafe { runtime.0.as_mut() }.unwrap();
    let c_str = unsafe { CStr::from_ptr(input) };
    let input_str = c_str.to_str().unwrap();

    match runtime.load_vtc(input_str) {
        Ok(_) => 0,
        Err(_) => -1,
    }
}

#[no_mangle]
pub extern "C" fn runtime_get_string(
    runtime: CRuntime,
    namespace: *const c_char,
    variable: *const c_char,
) -> *mut c_char {
    let runtime = unsafe { runtime.0.as_mut() }.unwrap();
    let namespace = unsafe { CStr::from_ptr(namespace) }.to_str().unwrap();
    let variable = unsafe { CStr::from_ptr(variable) }.to_str().unwrap();

    match runtime.get_string(namespace, variable) {
        Ok(s) => CString::new(s).unwrap().into_raw(),
        Err(_) => ptr::null_mut(),
    }
}

#[no_mangle]
pub extern "C" fn runtime_get_integer(
    runtime: CRuntime,
    namespace: *const c_char,
    variable: *const c_char,
    result: *mut i64,
) -> c_int {
    let runtime = unsafe { runtime.0.as_mut() }.unwrap();
    let namespace = unsafe { CStr::from_ptr(namespace) }.to_str().unwrap();
    let variable = unsafe { CStr::from_ptr(variable) }.to_str().unwrap();

    match runtime.get_value(namespace, variable, &[]) {
        Ok(value) => {
            if let Value::Number(Number::Integer(i)) = &*value {
                unsafe { *result = *i };
                0
            } else {
                -1
            }
        }
        Err(_) => -1,
    }
}

#[no_mangle]
pub extern "C" fn runtime_get_float(
    runtime: CRuntime,
    namespace: *const c_char,
    variable: *const c_char,
    result: *mut c_double,
) -> c_int {
    let runtime = unsafe { runtime.0.as_mut() }.unwrap();
    let namespace = unsafe { CStr::from_ptr(namespace) }.to_str().unwrap();
    let variable = unsafe { CStr::from_ptr(variable) }.to_str().unwrap();

    match runtime.get_value(namespace, variable, &[]) {
        Ok(value) => {
            if let Value::Number(Number::Float(f)) = &*value {
                unsafe { *result = *f };
                0
            } else {
                -1
            }
        }
        Err(_) => -1,
    }
}

#[no_mangle]
pub extern "C" fn runtime_get_boolean(
    runtime: CRuntime,
    namespace: *const c_char,
    variable: *const c_char,
    result: *mut bool,
) -> c_int {
    let runtime = unsafe { runtime.0.as_mut() }.unwrap();
    let namespace = unsafe { CStr::from_ptr(namespace) }.to_str().unwrap();
    let variable = unsafe { CStr::from_ptr(variable) }.to_str().unwrap();

    match runtime.get_value(namespace, variable, &[]) {
        Ok(value) => {
            if let Value::Boolean(b) = &*value {
                unsafe { *result = *b };
                0
            } else {
                -1
            }
        }
        Err(_) => -1,
    }
}

#[no_mangle]
pub extern "C" fn runtime_get_list(
    runtime: CRuntime,
    namespace: *const c_char,
    variable: *const c_char,
    result: *mut *mut Value,
    length: *mut usize,
) -> c_int {
    let runtime = unsafe { runtime.0.as_mut() }.unwrap();
    let namespace = unsafe { CStr::from_ptr(namespace) }.to_str().unwrap();
    let variable = unsafe { CStr::from_ptr(variable) }.to_str().unwrap();

    match runtime.get_value(namespace, variable, &[]) {
        Ok(value) => {
            if let Value::List(list) = &*value {
                // Clone the Vec<Value> and convert to boxed slice
                let values: Vec<Value> = list.iter().cloned().collect();
                let boxed_slice = values.into_boxed_slice();
                let raw_ptr = Box::into_raw(boxed_slice);
                unsafe {
                    *result = raw_ptr as *mut Value;
                    *length = (*raw_ptr).len();
                }
                0
            } else {
                -1
            }
        }
        Err(_) => -1,
    }
}

#[no_mangle]
pub extern "C" fn runtime_as_dict(
    runtime: CRuntime,
    namespace: *const c_char,
    variable: *const c_char,
) -> *mut c_void {
    let runtime = unsafe { runtime.0.as_mut() }.unwrap();
    let namespace = unsafe { CStr::from_ptr(namespace) }.to_str().unwrap();
    let variable = unsafe { CStr::from_ptr(variable) }.to_str().unwrap();

    match runtime.as_dict(namespace, variable) {
        Ok(dict) => {
            // Convert HashMap<String, Arc<Value>> to HashMap<String, Value>
            let converted: HashMap<String, Value> =
                dict.into_iter().map(|(k, v)| (k, (*v).clone())).collect();
            Box::into_raw(Box::new(converted)) as *mut c_void
        }
        Err(_) => ptr::null_mut(),
    }
}

#[no_mangle]
pub extern "C" fn runtime_flatten_list(
    runtime: CRuntime,
    namespace: *const c_char,
    variable: *const c_char,
    result: *mut *mut Value,
    length: *mut usize,
) -> c_int {
    let runtime = unsafe { runtime.0.as_mut() }.unwrap();
    let namespace = unsafe { CStr::from_ptr(namespace) }.to_str().unwrap();
    let variable = unsafe { CStr::from_ptr(variable) }.to_str().unwrap();

    match runtime.flatten_list(namespace, variable) {
        Ok(list) => {
            let boxed_slice = list.into_boxed_slice();
            let raw_ptr = Box::into_raw(boxed_slice);
            unsafe {
                *result = raw_ptr as *mut Value;
                *length = (*raw_ptr).len();
            }
            0
        }
        Err(_) => -1,
    }
}

#[no_mangle]
pub extern "C" fn runtime_list_namespaces(
    runtime: CRuntime,
    result: *mut *mut *mut c_char,
    length: *mut usize,
) -> c_int {
    let runtime = unsafe { runtime.0.as_mut() }.unwrap();
    let namespaces = runtime.list_namespaces();

    let c_strings: Vec<*mut c_char> = namespaces
        .into_iter()
        .map(|s| CString::new(s.to_string()).unwrap().into_raw())
        .collect();

    let boxed_slice = c_strings.into_boxed_slice();
    let raw_ptr = Box::into_raw(boxed_slice);

    unsafe {
        *result = raw_ptr as *mut *mut c_char;
        *length = (*raw_ptr).len();
    }

    0
}

#[no_mangle]
pub extern "C" fn runtime_list_variables(
    runtime: CRuntime,
    namespace: *const c_char,
    result: *mut *mut *mut c_char,
    length: *mut usize,
) -> c_int {
    let runtime = unsafe { runtime.0.as_mut() }.unwrap();
    let namespace = unsafe { CStr::from_ptr(namespace) }.to_str().unwrap();

    match runtime.list_variables(namespace) {
        Ok(variables) => {
            let c_strings: Vec<*mut c_char> = variables
                .into_iter()
                .map(|s| CString::new(s.to_string()).unwrap().into_raw())
                .collect();

            let boxed_slice = c_strings.into_boxed_slice();
            let raw_ptr = Box::into_raw(boxed_slice);

            unsafe {
                *result = raw_ptr as *mut *mut c_char;
                *length = (*raw_ptr).len();
            }
            0
        }
        Err(_) => -1,
    }
}

#[no_mangle]
pub extern "C" fn runtime_free(runtime: CRuntime) {
    if !runtime.0.is_null() {
        unsafe {
            drop(Box::from_raw(runtime.0));
        }
    }
}

#[no_mangle]
pub extern "C" fn runtime_free_string(s: *mut c_char) {
    if !s.is_null() {
        unsafe {
            drop(CString::from_raw(s));
        }
    }
}

#[no_mangle]
pub extern "C" fn runtime_free_string_array(arr: *mut *mut c_char, length: usize) {
    unsafe {
        if !arr.is_null() {
            let slice = slice::from_raw_parts_mut(arr, length);
            for &mut s in slice.iter_mut() {
                drop(CString::from_raw(s));
            }
            drop(Box::from_raw(slice));
        }
    }
}

#[no_mangle]
pub extern "C" fn runtime_free_value_array(arr: *mut Rc<Value>, length: usize) {
    if !arr.is_null() {
        unsafe {
            drop(Box::from_raw(slice::from_raw_parts_mut(arr, length)));
        }
    }
}

#[no_mangle]
pub extern "C" fn runtime_free_dict(dict: *mut c_void) {
    if !dict.is_null() {
        unsafe {
            drop(Box::from_raw(dict as *mut HashMap<String, Rc<Value>>));
        }
    }
}
