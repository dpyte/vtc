use std::ffi::CStr;
use std::os::raw::c_int;
use std::path::PathBuf;
use std::ptr;

use errno::{Errno, set_errno};
use libc::c_char;

use crate::runtime::error::RuntimeError;
use crate::runtime::runtime::Runtime;

#[no_mangle]
pub extern "C" fn vtc_runtime_new() -> *mut Runtime {
	let to_return = Runtime::new();

	Box::into_raw(Box::new(to_return))
}

#[no_mangle]
pub extern "C" fn vtc_runtime_from(path: *const c_char) -> *mut Runtime {
	unsafe {
		let path_str = CStr::from_ptr(path)
			.to_str()
			.unwrap_or_else(|_| panic!("The provided string could not be read."));

		let runtime_or_err = Runtime::from(PathBuf::from(path_str));

		match runtime_or_err {
			Ok(runtime) => Box::into_raw(Box::new(runtime)),
			Err(e) => {
				let errno_or_err = match e {
					RuntimeError::FileReadError {..} => 5,    // EIO
					RuntimeError::NamespaceNotFound {..} => 2, // ENOENT
					_ => 22,                         // EINVAL
				};

				set_errno(Errno(errno_or_err));

				ptr::null_mut()
			}
		}
	}
}

#[no_mangle]
pub extern "C" fn vtc_runtime_load_file(runtime: *mut Runtime, path: *const c_char) -> c_int {
	if path == ptr::null() || runtime == ptr::null_mut() {
		set_errno(Errno(22)); // EINVAL
		return -1;
	}

	let path_str = unsafe { CStr::from_ptr(path).to_str().expect("The provided string could not be read.") };
	let runtime_ref = unsafe{ &mut *runtime };

	match runtime_ref.load_file(PathBuf::from(path_str)) {
		Ok(_) => 0,
		Err(e) => {
			let errno_or_err = match e {
				RuntimeError::FileReadError {..} => 5,    // EIO
				RuntimeError::NamespaceNotFound {..} => 2, // ENOENT
				_ => 22,                         // EINVAL
			};


			set_errno(Errno(errno_or_err));

			-1
		}
	}
}

#[no_mangle]
pub extern "C" fn vtc_runtime_destroy(runtime: *mut Runtime) {
	if runtime != ptr::null_mut() {
		// drop the runtime
		unsafe { let _ = Box::from_raw(runtime); };
	}
}