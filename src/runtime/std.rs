use std::fmt;
use std::fmt::Formatter;
use std::sync::Arc;

use anyhow::{anyhow, Result};
use base64::{alphabet, Engine as _, engine::{self, general_purpose}};
use fnv::FnvHashMap;
use sha2::{Digest, Sha256};

use crate::value::Number;
use crate::value::Value;

pub type VtcFn = Box<dyn Fn(Vec<Arc<Value>>) -> Arc<Value> + Send + Sync>;

// Helper functions
mod helpers {
	use super::*;

	pub fn extract_number(value: &Arc<Value>) -> Number {
		match &**value {
			Value::Number(n) => n.clone(),
			_ => panic!("Expected a Number value"),
		}
	}

	pub fn extract_string(value: &Arc<Value>) -> String {
		match &**value {
			Value::String(s) => (**s).to_string(),
			_ => panic!("Expected a String value"),
		}
	}
}

// Arithmetic operations
mod arithmetic {
	use super::*;

	// Integer operations
	pub fn std_add_int(i1: Number, i2: Number) -> Number {
		match (i1, i2) {
			(Number::Integer(val1), Number::Integer(val2)) => Number::Integer(val1.wrapping_add(val2)),
			_ => panic!("Both inputs must be integers: {} and {}", i1, i2),
		}
	}

	pub fn std_sub_int(i1: Number, i2: Number) -> Number {
		match (i1, i2) {
			(Number::Integer(val1), Number::Integer(val2)) => Number::Integer(val1.wrapping_sub(val2)),
			_ => panic!("Both inputs must be integers: {} and {}", i1, i2),
		}
	}

	pub fn std_mul_int(i1: Number, i2: Number) -> Number {
		match (i1, i2) {
			(Number::Integer(val1), Number::Integer(val2)) => Number::Integer(val1.wrapping_mul(val2)),
			_ => panic!("Both inputs must be integers: {} and {}", i1, i2),
		}
	}

	pub fn std_div_int(i1: Number, i2: Number) -> Number {
		match (i1, i2) {
			(Number::Integer(val1), Number::Integer(val2)) if val2 != 0 => Number::Integer(val1 / val2),
			(Number::Integer(_), Number::Integer(0)) => panic!("Division by zero"),
			_ => panic!("Both inputs must be integers: {} and {}", i1, i2),
		}
	}

	pub fn std_mod_int(i1: Number, i2: Number) -> Number {
		match (i1, i2) {
			(Number::Integer(val1), Number::Integer(val2)) if val2 != 0 => Number::Integer(val1 % val2),
			(Number::Integer(_), Number::Integer(0)) => panic!("Modulo by zero"),
			_ => panic!("Both inputs must be integers: {} and {}", i1, i2),
		}
	}

	// Float operations
	pub fn std_add_float(f1: Number, f2: Number) -> Number {
		match (f1, f2) {
			(Number::Float(val1), Number::Float(val2)) => Number::Float(val1 + val2),
			_ => panic!("Both inputs must be floats: {} and {}", f1, f2),
		}
	}

	pub fn std_sub_float(f1: Number, f2: Number) -> Number {
		match (f1, f2) {
			(Number::Float(val1), Number::Float(val2)) => Number::Float(val1 - val2),
			_ => panic!("Both inputs must be floats: {} and {}", f1, f2),
		}
	}

	pub fn std_mul_float(f1: Number, f2: Number) -> Number {
		match (f1, f2) {
			(Number::Float(val1), Number::Float(val2)) => Number::Float(val1 * val2),
			_ => panic!("Both inputs must be floats: {} and {}", f1, f2),
		}
	}

	pub fn std_div_float(f1: Number, f2: Number) -> Number {
		match (f1, f2) {
			(Number::Float(val1), Number::Float(val2)) if val2 != 0.0 => Number::Float(val1 / val2),
			(Number::Float(_), Number::Float(val2)) if val2 == 0.0 => panic!("Division by zero"),
			_ => panic!("Both inputs must be floats: {} and {}", f1, f2),
		}
	}
}

// Conversion operations
mod conversion {
	use crate::value::Number;

	pub fn std_int_to_float(i: Number) -> Number {
		match i {
			Number::Integer(val) => Number::Float(val as f64),
			_ => panic!("Input must be an integer: {}", i),
		}
	}

	pub fn std_float_to_int(f: Number) -> Number {
		match f {
			Number::Float(val) => Number::Integer(val as i64),
			_ => panic!("Input must be a float: {:?}", f),
		}
	}
}

// Comparison operations
mod comparison {
	use crate::value::Number;

	pub fn std_eq(n1: Number, n2: Number) -> bool {
		match (n1, n2) {
			(Number::Integer(i1), Number::Integer(i2)) => i1 == i2,
			(Number::Float(f1), Number::Float(f2)) => f1 == f2,
			_ => false,
		}
	}

	pub fn std_lt(n1: Number, n2: Number) -> bool {
		match (n1, n2) {
			(Number::Integer(i1), Number::Integer(i2)) => i1 < i2,
			(Number::Float(f1), Number::Float(f2)) => f1 < f2,
			_ => panic!("Cannot compare different types: {} and {}", n1, n2),
		}
	}

	pub fn std_gt(n1: Number, n2: Number) -> bool {
		match (n1, n2) {
			(Number::Integer(i1), Number::Integer(i2)) => i1 > i2,
			(Number::Float(f1), Number::Float(f2)) => f1 > f2,
			_ => panic!("Cannot compare different types: {} and {}", n1, n2),
		}
	}
}

// Bitwise operations
mod bitwise {
	use crate::value::Number;

	pub fn std_bitwise_and(i1: Number, i2: Number) -> Number {
		match (i1, i2) {
			(Number::Integer(val1), Number::Integer(val2)) => Number::Integer(val1 & val2),
			_ => panic!("Both inputs must be integers: {} and {}", i1, i2),
		}
	}

	pub fn std_bitwise_or(i1: Number, i2: Number) -> Number {
		match (i1, i2) {
			(Number::Integer(val1), Number::Integer(val2)) => Number::Integer(val1 | val2),
			_ => panic!("Both inputs must be integers: {} and {}", i1, i2),
		}
	}

	pub fn std_bitwise_xor(i1: Number, i2: Number) -> Number {
		match (i1, i2) {
			(Number::Integer(val1), Number::Integer(val2)) => Number::Integer(val1 ^ val2),
			_ => panic!("Both inputs must be integers: {} and {}", i1, i2),
		}
	}

	pub fn std_bitwise_not(i: Number) -> Number {
		match i {
			Number::Integer(val) => Number::Integer(!val),
			_ => panic!("Input must be an integer: {}", i),
		}
	}
}

// String operations
mod string_ops {
	use crate::value::Value;

	use super::*;

	pub fn std_to_uppercase(args: Vec<Arc<Value>>) -> Arc<Value> {
		if args.len() != 1 {
			panic!("std_to_uppercase expects 1 argument");
		}
		let s = helpers::extract_string(&args[0]);
		Arc::new(Value::String(s.to_uppercase()))
	}

	pub fn std_to_lowercase(args: Vec<Arc<Value>>) -> Arc<Value> {
		if args.len() != 1 {
			panic!("std_to_lowercase expects 1 argument");
		}
		let s = helpers::extract_string(&args[0]);
		Arc::new(Value::String(s.to_lowercase()))
	}

	pub fn std_substring(args: Vec<Arc<Value>>) -> Arc<Value> {
		if args.len() != 3 {
			panic!("std_substring expects 3 arguments");
		}
		let s = helpers::extract_string(&args[0]);
		let start = helpers::extract_number(&args[1]);
		let end = helpers::extract_number(&args[2]);

		if let (Number::Integer(start), Number::Integer(end)) = (start, end) {
			let start = start as usize;
			let end = end as usize;
			if start > end || end > s.len() {
				panic!("Invalid range for substring");
			}
			Arc::new(Value::String(s[start..end].to_string()))
		} else {
			panic!("Start and end indices must be integers");
		}
	}

	/// Appends two or more strings together
	pub fn std_concat(args: Vec<Arc<Value>>) -> Arc<Value> {
		let mut result = String::new();
		for arg in args {
			result.push_str(&helpers::extract_string(&arg));
		}
		Arc::new(Value::String(result))
	}

	pub fn std_replace(args: Vec<Arc<Value>>) -> Arc<Value> {
		if args.len() != 3 {
			panic!("std_replace expects 3 arguments");
		}
		let s = helpers::extract_string(&args[0]);
		let from = helpers::extract_string(&args[1]);
		let to = helpers::extract_string(&args[2]);
		Arc::new(Value::String(s.replace(&from, &to)))
	}
}

// Advanced operations
mod advanced_ops {
	use super::*;

	const CUSTOM_ENGINE: engine::GeneralPurpose = engine::GeneralPurpose::new(
		&alphabet::URL_SAFE, general_purpose::NO_PAD);

	pub fn std_base64_encode(args: Vec<Arc<Value>>) -> Arc<Value> {
		if args.len() != 1 {
			panic!("std_base64_encode expects 1 argument");
		}
		let s = helpers::extract_string(&args[0]);
		Arc::new(Value::String(CUSTOM_ENGINE.encode(s)))
	}

	pub fn std_base64_decode(args: Vec<Arc<Value>>) -> Arc<Value> {
		if args.len() != 1 {
			panic!("std_base64_decode expects 1 argument");
		}
		let s = helpers::extract_string(&args[0]);
		match CUSTOM_ENGINE.decode(s) {
			Ok(decoded) => match String::from_utf8(decoded) {
				Ok(decoded_str) => Arc::new(Value::String(decoded_str)),
				Err(_) => panic!("Failed to convert decoded bytes to UTF-8 string"),
			},
			Err(_) => panic!("Failed to decode base64 string"),
		}
	}

	pub fn std_hash(args: Vec<Arc<Value>>) -> Arc<Value> {
		if args.len() != 2 {
			panic!("std_hash expects 2 arguments");
		}
		let s = helpers::extract_string(&args[0]);
		let algorithm = helpers::extract_string(&args[1]);
		match algorithm.as_str() {
			"sha256" => {
				let mut hasher = Sha256::new();
				hasher.update(s.as_bytes());
				let result = hasher.finalize();
				Arc::new(Value::String(format!("{:x}", result)))
			},
			_ => panic!("Unsupported hash algorithm: {}", algorithm),
		}
	}
}

// Control flow
mod control_flow {
	use super::*;

	pub fn std_if(args: Vec<Arc<Value>>) -> Arc<Value> {
		if args.len() != 3 {
			panic!("std_if expects 3 arguments: condition, true_value, false_value");
		}
		match &*args[0] {
			Value::Boolean(condition) => {
				if *condition {
					args[1].clone()
				} else {
					args[2].clone()
				}
			},
			_ => panic!("First argument of std_if must be a boolean"),
		}
	}

	pub fn std_try(args: Vec<Arc<Value>>) -> Arc<Value> {
		if args.len() != 2 {
			panic!("std_try expects 2 arguments: expression, default_value");
		}
		// In a real implementation, you'd want to actually try evaluating the first argument
		// and return the second if it fails. For now, we'll just return the first argument.
		args[0].clone()
	}
}

// Wrapper functions for arithmetic operations
macro_rules! create_arithmetic_wrapper {
    ($name:ident, $func:path) => {
        fn $name(args: Vec<Arc<Value>>) -> Arc<Value> {
            if args.len() != 2 {
                panic!(concat!(stringify!($name), " expects 2 arguments"));
            }
            let n1 = helpers::extract_number(&args[0]);
            let n2 = helpers::extract_number(&args[1]);
            Arc::new(Value::Number($func(n1, n2)))
        }
    };
}

create_arithmetic_wrapper!(std_add_int_wrapper, arithmetic::std_add_int);
create_arithmetic_wrapper!(std_sub_int_wrapper, arithmetic::std_sub_int);
create_arithmetic_wrapper!(std_mul_int_wrapper, arithmetic::std_mul_int);
create_arithmetic_wrapper!(std_div_int_wrapper, arithmetic::std_div_int);
create_arithmetic_wrapper!(std_mod_int_wrapper, arithmetic::std_mod_int);
create_arithmetic_wrapper!(std_add_float_wrapper, arithmetic::std_add_float);
create_arithmetic_wrapper!(std_sub_float_wrapper, arithmetic::std_sub_float);
create_arithmetic_wrapper!(std_mul_float_wrapper, arithmetic::std_mul_float);
create_arithmetic_wrapper!(std_div_float_wrapper, arithmetic::std_div_float);

// Wrapper functions for conversion operations
fn std_int_to_float_wrapper(args: Vec<Arc<Value>>) -> Arc<Value> {
	if args.len() != 1 {
		panic!("std_int_to_float expects 1 argument");
	}
	let n = helpers::extract_number(&args[0]);
	Arc::new(Value::Number(conversion::std_int_to_float(n)))
}

fn std_float_to_int_wrapper(args: Vec<Arc<Value>>) -> Arc<Value> {
	if args.len() != 1 {
		panic!("std_float_to_int expects 1 argument");
	}
	let n = helpers::extract_number(&args[0]);
	Arc::new(Value::Number(conversion::std_float_to_int(n)))
}

// Wrapper functions for comparison operations
fn std_eq_wrapper(args: Vec<Arc<Value>>) -> Arc<Value> {
	if args.len() != 2 {
		panic!("std_eq expects 2 arguments");
	}
	let n1 = helpers::extract_number(&args[0]);
	let n2 = helpers::extract_number(&args[1]);
	Arc::new(Value::Boolean(comparison::std_eq(n1, n2)))
}

fn std_lt_wrapper(args: Vec<Arc<Value>>) -> Arc<Value> {
	if args.len() != 2 {
		panic!("std_lt expects 2 arguments");
	}
	let n1 = helpers::extract_number(&args[0]);
	let n2 = helpers::extract_number(&args[1]);
	Arc::new(Value::Boolean(comparison::std_lt(n1, n2)))
}

fn std_gt_wrapper(args: Vec<Arc<Value>>) -> Arc<Value> {
	if args.len() != 2 {
		panic!("std_gt expects 2 arguments");
	}
	let n1 = helpers::extract_number(&args[0]);
	let n2 = helpers::extract_number(&args[1]);
	Arc::new(Value::Boolean(comparison::std_gt(n1, n2)))
}

// Wrapper functions for bitwise operations
create_arithmetic_wrapper!(std_bitwise_and_wrapper, bitwise::std_bitwise_and);
create_arithmetic_wrapper!(std_bitwise_or_wrapper, bitwise::std_bitwise_or);
create_arithmetic_wrapper!(std_bitwise_xor_wrapper, bitwise::std_bitwise_xor);

fn std_bitwise_not_wrapper(args: Vec<Arc<Value>>) -> Arc<Value> {
	if args.len() != 1 {
		panic!("std_bitwise_not expects 1 argument");
	}
	let n = helpers::extract_number(&args[0]);
	Arc::new(Value::Number(bitwise::std_bitwise_not(n)))
}

/// StdLibLoader
/// Load and manage standard library functions
pub struct StdLibLoader {
	loadable: FnvHashMap<String, VtcFn>,
}

#[macro_export]
macro_rules! register_function {
    ($loader:expr, $name:expr, $func:expr) => {
	    $loader.register_function($name.to_string(), Box::new($func) as VtcFn);
    };
}

impl StdLibLoader {
	pub fn new() -> Self {
		let mut loadable = FnvHashMap::default();

		// Arithmetic operations
		loadable.insert("std_add_int".to_string(), Box::new(std_add_int_wrapper) as VtcFn);
		loadable.insert("std_sub_int".to_string(), Box::new(std_sub_int_wrapper) as VtcFn);
		loadable.insert("std_mul_int".to_string(), Box::new(std_mul_int_wrapper) as VtcFn);
		loadable.insert("std_div_int".to_string(), Box::new(std_div_int_wrapper) as VtcFn);
		loadable.insert("std_mod_int".to_string(), Box::new(std_mod_int_wrapper) as VtcFn);
		loadable.insert("std_add_float".to_string(), Box::new(std_add_float_wrapper) as VtcFn);
		loadable.insert("std_sub_float".to_string(), Box::new(std_sub_float_wrapper) as VtcFn);
		loadable.insert("std_mul_float".to_string(), Box::new(std_mul_float_wrapper) as VtcFn);
		loadable.insert("std_div_float".to_string(), Box::new(std_div_float_wrapper) as VtcFn);

		// Conversion operations
		loadable.insert("std_int_to_float".to_string(), Box::new(std_int_to_float_wrapper) as VtcFn);
		loadable.insert("std_float_to_int".to_string(), Box::new(std_float_to_int_wrapper) as VtcFn);

		// Comparison operations
		loadable.insert("std_eq".to_string(), Box::new(std_eq_wrapper) as VtcFn);
		loadable.insert("std_lt".to_string(), Box::new(std_lt_wrapper) as VtcFn);
		loadable.insert("std_gt".to_string(), Box::new(std_gt_wrapper) as VtcFn);

		// Bitwise operations
		loadable.insert("std_bitwise_and".to_string(), Box::new(std_bitwise_and_wrapper) as VtcFn);
		loadable.insert("std_bitwise_or".to_string(), Box::new(std_bitwise_or_wrapper) as VtcFn);
		loadable.insert("std_bitwise_xor".to_string(), Box::new(std_bitwise_xor_wrapper) as VtcFn);
		loadable.insert("std_bitwise_not".to_string(), Box::new(std_bitwise_not_wrapper) as VtcFn);

		// String operations
		loadable.insert("std_to_uppercase".to_string(), Box::new(string_ops::std_to_uppercase) as VtcFn);
		loadable.insert("std_to_lowercase".to_string(), Box::new(string_ops::std_to_lowercase) as VtcFn);
		loadable.insert("std_substring".to_string(), Box::new(string_ops::std_substring) as VtcFn);
		loadable.insert("std_concat".to_string(), Box::new(string_ops::std_concat) as VtcFn);
		loadable.insert("std_replace".to_string(), Box::new(string_ops::std_replace) as VtcFn);

		// Advanced operations
		loadable.insert("std_base64_encode".to_string(), Box::new(advanced_ops::std_base64_encode) as VtcFn);
		loadable.insert("std_base64_decode".to_string(), Box::new(advanced_ops::std_base64_decode) as VtcFn);
		loadable.insert("std_hash".to_string(), Box::new(advanced_ops::std_hash) as VtcFn);

		// Control flow
		loadable.insert("std_if".to_string(), Box::new(control_flow::std_if) as VtcFn);
		loadable.insert("std_try".to_string(), Box::new(control_flow::std_try) as VtcFn);

		Self { loadable }
	}

	pub fn get_function(&self, name: &str) -> Option<&VtcFn> {
		self.loadable.get(name)
	}

	pub fn register_function(&mut self, name: String, function: VtcFn) -> Result<()> {
		if name.starts_with("std") {
			return Err(anyhow!("User defined functions cannot start with `std`.".to_string()))
		}
		self.loadable.insert(name, function);
		Ok(())
	}
}

impl fmt::Debug for StdLibLoader {
	fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
		f.debug_struct("StdLibLoader").field("loadable", &self.loadable.keys().collect::<Vec<_>>())
			.finish()
	}
}
