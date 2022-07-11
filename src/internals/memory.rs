
///
/// Preface:
/// A large component of vtc file is that it allows using values by pointer or reference which enables
/// it to serialize lucrative datastructures.
/// For example, following representation in C99 and C++14 can be serialized to following in vtc.
///
/// C99:
///     int a = 99;
///     int *data = &a;
///
/// VTC:
///     @...:
///         $a := 99,
///         $data := [ &a ]
///
/// CXX14:
///     const auto a1, a2, a3, a4 = 1, 2, 3;
///     const auto a = std::vector<std::uint32*>(&a1, &a2, &a3);
///
/// VTC:
///     @cxx:
///         $arrays := [ 1, 2, 3 ],
///         $data   := ![uint32]{%cxx.arrays[..]}
///
/// Expression `$data` will expand into:
///     $data := ![uint32]{
///         &arrays->0,
///         &arrays->1,
///         &arrays->2,
///     },
///
/// Difference b/w ...->1 and ...->(1):
///    Access by Pointer: Any value following the arrow hints a pointer value/ memory location.
///    Access by Value: Any value enclosed in set of single parenthesis following the arrow.
///
/// Task:
///     To design memory framework that allows capturing such values
///     - static values/ predefined values can be stored in its own segment
///     - runtime values can be stored in a `container`
///

use std::alloc::{alloc, dealloc, Layout};

/// Base defines a basic structure on which other datatypes will rely on
pub struct Base<T> {
	layout: Layout,
	alloc_ptr: *mut T,
	release: bool,
}

///
/// Construct and initialize the memory block using std::alloc::layout as backend.
/// Returns BaseMemErr in case the underlying structure fails
///
impl<T> Base<T> {
	/// Initialize value based on defined parameters
	pub unsafe fn new() -> Self {
		let layout = Layout::new::<T>();
		let alloc_ptr = alloc(layout) as *mut T;
		Self { layout, alloc_ptr, release: false }
	}

	/// Release and invalidate the block
	pub fn release(mut self) {
		self.release = true;
		unsafe {
			dealloc(self.alloc_ptr as *mut u8, self.layout);
		}
	}

	/// Returns validity i.e., status of the block
	pub fn is_valid(&self) -> bool {
		// Return inverse of the release mode
		!self.release
	}
}

