#![allow(dead_code)]

use std::sync::Arc;

use crate::value::Value;

pub mod cffi;
pub mod parser;
pub mod runtime;
pub mod value;

pub const SMALL_VEC_SIZE: usize = 512;

pub type VtcFnArg = Vec<Arc<Value>>;
pub type VtcFnRet = Arc<Value>;
