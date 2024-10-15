#![allow(dead_code)]

pub mod parser;
pub mod runtime;
pub mod cffi;
pub mod value;

pub const SMALL_VEC_SIZE: usize = 64;
