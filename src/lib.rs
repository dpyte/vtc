use lasso::Rodeo;

pub mod parser;
pub mod runtime;
pub mod ffi;

pub type Interner = Rodeo;

pub const SMALL_VEC_SIZE: usize = 64;
