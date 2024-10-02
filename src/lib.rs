use lasso::{Rodeo, Spur};

pub mod parser;
pub mod runtime;

pub const SMALL_VEC_SIZE: usize = 64;

pub type Interner = Rodeo;
type StringKey = Spur;