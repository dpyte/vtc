#![allow(dead_code)]

use std::rc::Rc;
use crate::value::Value;

pub mod parser;
pub mod runtime;
pub mod cffi;
pub mod value;

pub const SMALL_VEC_SIZE: usize = 64;


pub type VtcFnArg = Vec<Rc<Value>>;
pub type VtcFnRet = Rc<Value>;
