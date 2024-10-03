use std::collections::HashMap;
use std::rc::Rc;

use base64::{decode, encode};
use sha2::{Digest, Sha256};

use crate::{Number, Value};

pub type VtcFn = Box<dyn Fn(Vec<Rc<Value>>) -> Rc<Value>>;


// Helper functions
fn extract_number(value: &Rc<Value>) -> Number {
	match &**value {
		Value::Number(n) => n.clone(),
		_ => panic!("Expected a Number value"),
	}
}

fn extract_string(value: &Rc<Value>) -> String {
	match &**value {
		Value::String(s) => (**s).clone(),
		_ => panic!("Expected a String value"),
	}
}

// Wrapper functions for arithmetic operations
macro_rules! create_arithmetic_wrapper {
    ($name:ident, $func:ident) => {
        fn $name(args: Vec<Rc<Value>>) -> Rc<Value> {
            if args.len() != 2 {
                panic!(concat!(stringify!($func), " expects 2 arguments"));
            }
            let n1 = extract_number(&args[0]);
            let n2 = extract_number(&args[1]);
            Rc::new(Value::Number($func(n1, n2)))
        }
    };
}

create_arithmetic_wrapper!(std_add_int_wrapper, std_add_int);
create_arithmetic_wrapper!(std_sub_int_wrapper, std_sub_int);
create_arithmetic_wrapper!(std_mul_int_wrapper, std_mul_int);
create_arithmetic_wrapper!(std_div_int_wrapper, std_div_int);
create_arithmetic_wrapper!(std_mod_int_wrapper, std_mod_int);
create_arithmetic_wrapper!(std_add_float_wrapper, std_add_float);
create_arithmetic_wrapper!(std_sub_float_wrapper, std_sub_float);
create_arithmetic_wrapper!(std_mul_float_wrapper, std_mul_float);
create_arithmetic_wrapper!(std_div_float_wrapper, std_div_float);

// Wrapper functions for conversion operations
fn std_int_to_float_wrapper(args: Vec<Rc<Value>>) -> Rc<Value> {
	if args.len() != 1 {
		panic!("std_int_to_float expects 1 argument");
	}
	let n = extract_number(&args[0]);
	Rc::new(Value::Number(std_int_to_float(n)))
}

fn std_float_to_int_wrapper(args: Vec<Rc<Value>>) -> Rc<Value> {
	if args.len() != 1 {
		panic!("std_float_to_int expects 1 argument");
	}
	let n = extract_number(&args[0]);
	Rc::new(Value::Number(std_float_to_int(n)))
}

// Wrapper functions for bitwise operations
create_arithmetic_wrapper!(std_bitwise_and_wrapper, std_bitwise_and);
create_arithmetic_wrapper!(std_bitwise_or_wrapper, std_bitwise_or);
create_arithmetic_wrapper!(std_bitwise_xor_wrapper, std_bitwise_xor);

// String operations
fn std_to_uppercase(args: Vec<Rc<Value>>) -> Rc<Value> {
	if args.len() != 1 {
		panic!("std_to_uppercase expects 1 argument");
	}
	let s = extract_string(&args[0]);
	Rc::new(Value::String(Rc::new(s.to_uppercase())))
}

fn std_to_lowercase(args: Vec<Rc<Value>>) -> Rc<Value> {
	if args.len() != 1 {
		panic!("std_to_lowercase expects 1 argument");
	}
	let s = extract_string(&args[0]);
	Rc::new(Value::String(Rc::new(s.to_lowercase())))
}

fn std_substring(args: Vec<Rc<Value>>) -> Rc<Value> {
	if args.len() != 3 {
		panic!("std_substring expects 3 arguments");
	}
	let s = extract_string(&args[0]);
	let start = extract_number(&args[1]);
	let end = extract_number(&args[2]);

	if let (Number::Integer(start), Number::Integer(end)) = (start, end) {
		let start = start as usize;
		let end = end as usize;
		if start > end || end > s.len() {
			panic!("Invalid range for substring");
		}
		Rc::new(Value::String(Rc::new(s[start..end].to_string())))
	} else {
		panic!("Start and end indices must be integers");
	}
}

fn std_concat(args: Vec<Rc<Value>>) -> Rc<Value> {
	let mut result = String::new();
	for arg in args {
		result.push_str(&extract_string(&arg));
	}
	Rc::new(Value::String(Rc::new(result)))
}

fn std_replace(args: Vec<Rc<Value>>) -> Rc<Value> {
	if args.len() != 3 {
		panic!("std_replace expects 3 arguments");
	}
	let s = extract_string(&args[0]);
	let from = extract_string(&args[1]);
	let to = extract_string(&args[2]);
	Rc::new(Value::String(Rc::new(s.replace(&from, &to))))
}

// Advanced operations
fn std_base64_encode(args: Vec<Rc<Value>>) -> Rc<Value> {
	if args.len() != 1 {
		panic!("std_base64_encode expects 1 argument");
	}
	let s = extract_string(&args[0]);
	Rc::new(Value::String(Rc::new(encode(s))))
}

fn std_base64_decode(args: Vec<Rc<Value>>) -> Rc<Value> {
	if args.len() != 1 {
		panic!("std_base64_decode expects 1 argument");
	}
	let s = extract_string(&args[0]);
	match decode(s) {
		Ok(decoded) => match String::from_utf8(decoded) {
			Ok(decoded_str) => Rc::new(Value::String(Rc::new(decoded_str))),
			Err(_) => panic!("Failed to convert decoded bytes to UTF-8 string"),
		},
		Err(_) => panic!("Failed to decode base64 string"),
	}
}

fn std_hash(args: Vec<Rc<Value>>) -> Rc<Value> {
	if args.len() != 2 {
		panic!("std_hash expects 2 arguments");
	}
	let s = extract_string(&args[0]);
	let algorithm = extract_string(&args[1]);
	match algorithm.as_str() {
		"sha256" => {
			let mut hasher = Sha256::new();
			hasher.update(s.as_bytes());
			let result = hasher.finalize();
			Rc::new(Value::String(Rc::new(format!("{:x}", result))))
		},
		_ => panic!("Unsupported hash algorithm: {}", algorithm),
	}
}

// Control flow
fn std_if(args: Vec<Rc<Value>>) -> Rc<Value> {
	if args.len() != 3 {
		panic!("std_if expects 3 arguments: condition, true_value, false_value");
	}
	match &*args[0] {
		Value::Boolean(condition) => {
			if *condition {
				Rc::clone(&args[1])
			} else {
				Rc::clone(&args[2])
			}
		},
		_ => panic!("First argument of std_if must be a boolean"),
	}
}

fn std_try(args: Vec<Rc<Value>>) -> Rc<Value> {
	if args.len() != 2 {
		panic!("std_try expects 2 arguments: expression, default_value");
	}
	// In a real implementation, you'd want to actually try evaluating the first argument
	// and return the second if it fails. For now, we'll just return the first argument.
	Rc::clone(&args[0])
}


/// StdLibLoader
/// Load and manage standard library functions
pub struct StdLibLoader {
	loadable: HashMap<String, VtcFn>,
}

impl StdLibLoader {
	pub fn new() -> Self {
		let mut loadable = HashMap::new();

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
		loadable.insert("std_to_uppercase".to_string(), Box::new(std_to_uppercase) as VtcFn);
		loadable.insert("std_to_lowercase".to_string(), Box::new(std_to_lowercase) as VtcFn);
		loadable.insert("std_substring".to_string(), Box::new(std_substring) as VtcFn);
		loadable.insert("std_concat".to_string(), Box::new(std_concat) as VtcFn);
		loadable.insert("std_replace".to_string(), Box::new(std_replace) as VtcFn);

		// Advanced operations
		loadable.insert("std_base64_encode".to_string(), Box::new(std_base64_encode) as VtcFn);
		loadable.insert("std_base64_decode".to_string(), Box::new(std_base64_decode) as VtcFn);
		loadable.insert("std_hash".to_string(), Box::new(std_hash) as VtcFn);

		// Control flow
		loadable.insert("std_if".to_string(), Box::new(std_if) as VtcFn);
		loadable.insert("std_try".to_string(), Box::new(std_try) as VtcFn);

		Self { loadable }
	}

	pub fn get_function(&self, name: &str) -> Option<&VtcFn> {
		self.loadable.get(name)
	}
}


/// Performs addition operation on two integers.
///
/// # Arguments
///
/// * `i1` - The first integer to be added.
/// * `i2` - The second integer to be added.
///
/// # Returns
///
/// The result of adding `i1` and `i2` as an integer.
///
/// # Panics
///
/// This function will panic if `i1` or `i2` is not of type `Number::Integer`.
// Integer operations
pub fn std_add_int(i1: Number, i2: Number) -> Number {
	match (i1, i2) {
		(Number::Integer(val1), Number::Integer(val2)) => Number::Integer(val1.wrapping_add(val2)),
		_ => panic!("Both inputs must be integers: {} and {}", i1, i2),
	}
}

/// Subtracts two integers and returns the result as an integer.
///
/// # Arguments
///
/// * `i1` - The first integer.
/// * `i2` - The second integer.
///
/// # Returns
///
/// The difference of `i1` and `i2` as an integer.
///
/// # Panics
///
/// Panics if either `i1` or `i2` is not an integer.
pub fn std_sub_int(i1: Number, i2: Number) -> Number {
	match (i1, i2) {
		(Number::Integer(val1), Number::Integer(val2)) => Number::Integer(val1.wrapping_sub(val2)),
		_ => panic!("Both inputs must be integers: {} and {}", i1, i2),
	}
}

/// Multiplies two integers.
///
/// # Arguments
///
/// * `i1` - The first integer to multiply.
/// * `i2` - The second integer to multiply.
///
/// # Returns
///
/// The product of the two input integers.
///
/// # Panics
///
/// This function will panic if either `i1` or `i2` is not an `Number::Integer`.
pub fn std_mul_int(i1: Number, i2: Number) -> Number {
	match (i1, i2) {
		(Number::Integer(val1), Number::Integer(val2)) => Number::Integer(val1.wrapping_mul(val2)),
		_ => panic!("Both inputs must be integers: {} and {}", i1, i2),
	}
}

/// This function performs integer division between two `Number` values.
///
/// # Arguments
///
/// * `i1` - The first `Number` value to be divided.
/// * `i2` - The second `Number` value to divide by.
///
/// # Panics
///
/// - If both `i1` and `i2` are not of type `Number::Integer`.
/// - If `i2` is of type `Number::Integer` and its value is zero.
///
/// # Returns
///
/// The result of integer division as a `Number` value.
/// ```
pub fn std_div_int(i1: Number, i2: Number) -> Number {
	match (i1, i2) {
		(Number::Integer(val1), Number::Integer(val2)) if val2 != 0 => Number::Integer(val1 / val2),
		(Number::Integer(_), Number::Integer(0)) => panic!("Division by zero"),
		_ => panic!("Both inputs must be integers: {} and {}", i1, i2),
	}
}

/// Calculates the modulo of two integer numbers.
///
/// # Arguments
///
/// * `i1` - The first integer number.
/// * `i2` - The second integer number.
///
/// # Panics
///
/// Panics if `i2` is zero or if either `i1` or `i2` is not an integer number.
///
/// # Returns
///
/// The result of `i1 % i2` if both `i1` and `i2` are integer numbers and `i2` is not zero.
pub fn std_mod_int(i1: Number, i2: Number) -> Number {
	match (i1, i2) {
		(Number::Integer(val1), Number::Integer(val2)) if val2 != 0 => Number::Integer(val1 % val2),
		(Number::Integer(_), Number::Integer(0)) => panic!("Modulo by zero"),
		_ => panic!("Both inputs must be integers: {} and {}", i1, i2),
	}
}

/// Performs standard addition operation on two float numbers.
///
/// # Arguments
///
/// * `f1` - The first float number to be added.
/// * `f2` - The second float number to be added.
///
/// # Returns
///
/// The sum of `f1` and `f2`.
///
/// # Panics
///
/// Panics if both `f1` and `f2` are not of type `Number::Float`.
// Float operations
pub fn std_add_float(f1: Number, f2: Number) -> Number {
	match (f1, f2) {
		(Number::Float(val1), Number::Float(val2)) => Number::Float(val1 + val2),
		_ => panic!("Both inputs must be floats: {} and {}", f1, f2),
	}
}

/// Subtracts two floating point numbers.
///
/// # Arguments
///
/// * `f1` - The first floating point number.
/// * `f2` - The second floating point number.
///
/// # Panics
///
/// Panics if either `f1` or `f2` is not a `Number::Float`.
///
/// # Returns
///
/// The difference between `f1` and `f2`.
pub fn std_sub_float(f1: Number, f2: Number) -> Number {
	match (f1, f2) {
		(Number::Float(val1), Number::Float(val2)) => Number::Float(val1 - val2),
		_ => panic!("Both inputs must be floats: {} and {}", f1, f2),
	}
}

/// Multiplies two floating-point numbers.
///
/// # Arguments
///
/// * `f1` - The first floating-point number to multiply.
/// * `f2` - The second floating-point number to multiply.
///
/// # Returns
///
/// The product of `f1` and `f2`.
///
/// # Panics
///
/// This function panics if either `f1` or `f2` is not a floating-point number.
///
pub fn std_mul_float(f1: Number, f2: Number) -> Number {
	match (f1, f2) {
		(Number::Float(val1), Number::Float(val2)) => Number::Float(val1 * val2),
		_ => panic!("Both inputs must be floats: {} and {}", f1, f2),
	}
}

/// Divides two floating-point numbers and returns the result.
///
/// # Arguments
///
/// * `f1` - The first floating-point number to be divided.
/// * `f2` - The second floating-point number to be divided.
///
/// # Panics
///
/// This function will panic under the following conditions:
///
/// * If both `f1` and `f2` are not floating-point numbers.
/// * If `f2` is zero.
///
/// # Returns
///
/// The result of dividing `f1` by `f2`.
pub fn std_div_float(f1: Number, f2: Number) -> Number {
	match (f1, f2) {
		(Number::Float(val1), Number::Float(val2)) if val2 != 0.0 => Number::Float(val1 / val2),
		(Number::Float(_), Number::Float(val2)) if val2 == 0.0 => panic!("Division by zero"),
		_ => panic!("Both inputs must be floats: {} and {}", f1, f2),
	}
}

/// Converts a standard integer to a floating-point number.
///
/// # Arguments
///
/// * `i` - A `Number` enum that represents an integer value.
///
/// # Returns
///
/// A `Number` enum that represents the input value as a floating-point number.
///
/// # Panics
///
/// Panics if the input is not an integer.
// Conversion functions
pub fn std_int_to_float(i: Number) -> Number {
	match i {
		Number::Integer(val) => Number::Float(val as f64),
		_ => panic!("Input must be an integer: {}", i),
	}
}

/// Converts a number from a floating-point representation to an integer representation.
///
/// # Arguments
///
/// * `f` - The number to be converted, represented as a `Number`.
///
/// # Returns
///
/// The converted number, represented as a `Number::Integer`.
///
/// # Panics
///
/// Panics if the input number is not a float.
pub fn std_float_to_int(f: Number) -> Number {
	match f {
		Number::Float(val) => Number::Integer(val as i64),
		_ => panic!("Input must be a float: {}", f),
	}
}

// Comparison functions

// Wrapper function for std_eq
fn std_eq_wrapper(args: Vec<Rc<Value>>) -> Rc<Value> {
	if args.len() != 2 {
		panic!("std_eq expects 2 arguments");
	}
	let n1 = extract_number(&args[0]);
	let n2 = extract_number(&args[1]);
	Rc::new(Value::Boolean(std_eq(n1, n2)))
}

// Wrapper function for std_lt
fn std_lt_wrapper(args: Vec<Rc<Value>>) -> Rc<Value> {
	if args.len() != 2 {
		panic!("std_lt expects 2 arguments");
	}
	let n1 = extract_number(&args[0]);
	let n2 = extract_number(&args[1]);
	Rc::new(Value::Boolean(std_lt(n1, n2)))
}

// Wrapper function for std_gt
fn std_gt_wrapper(args: Vec<Rc<Value>>) -> Rc<Value> {
	if args.len() != 2 {
		panic!("std_gt expects 2 arguments");
	}
	let n1 = extract_number(&args[0]);
	let n2 = extract_number(&args[1]);
	Rc::new(Value::Boolean(std_gt(n1, n2)))
}

// Wrapper function for std_bitwise_not
fn std_bitwise_not_wrapper(args: Vec<Rc<Value>>) -> Rc<Value> {
	if args.len() != 1 {
		panic!("std_bitwise_not expects 1 argument");
	}
	let n = extract_number(&args[0]);
	Rc::new(Value::Number(std_bitwise_not(n)))
}

/// Compares two numbers for equality.
///
/// # Parameters
///
/// - `n1`: The first number to compare.
/// - `n2`: The second number to compare.
///
/// # Returns
///
/// Returns `true` if the numbers are equal, otherwise `false`.
pub fn std_eq(n1: Number, n2: Number) -> bool {
	match (n1, n2) {
		(Number::Integer(i1), Number::Integer(i2)) => i1 == i2,
		(Number::Float(f1), Number::Float(f2)) => f1 == f2,
		_ => false,
	}
}

/// Compares two numbers and returns `true` if the first number is less than the second number.
///
/// # Arguments
///
/// * `n1` - The first number.
/// * `n2` - The second number.
///
/// # Panics
///
/// This function will panic if the two numbers have different types.
pub fn std_lt(n1: Number, n2: Number) -> bool {
	match (n1, n2) {
		(Number::Integer(i1), Number::Integer(i2)) => i1 < i2,
		(Number::Float(f1), Number::Float(f2)) => f1 < f2,
		_ => panic!("Cannot compare different types: {} and {}", n1, n2),
	}
}

/// Compares two numbers and returns true if the first number is greater than the second number.
///
/// # Arguments
///
/// * `n1` - The first number to compare.
/// * `n2` - The second number to compare.
///
/// # Panics
///
/// This function will panic if the two numbers have different types.
pub fn std_gt(n1: Number, n2: Number) -> bool {
	match (n1, n2) {
		(Number::Integer(i1), Number::Integer(i2)) => i1 > i2,
		(Number::Float(f1), Number::Float(f2)) => f1 > f2,
		_ => panic!("Cannot compare different types: {} and {}", n1, n2),
	}
}

/// Performs a bitwise AND operation between two integers.
///
/// # Arguments
///
/// * `i1` - The first integer operand.
/// * `i2` - The second integer operand.
///
/// # Panics
///
/// Panics if either `i1` or `i2` is not an integer.
///
/// # Returns
///
/// The result of the bitwise AND operation as an integer.
// Bitwise operations (for integers only)
pub fn std_bitwise_and(i1: Number, i2: Number) -> Number {
	match (i1, i2) {
		(Number::Integer(val1), Number::Integer(val2)) => Number::Integer(val1 & val2),
		_ => panic!("Both inputs must be integers: {} and {}", i1, i2),
	}
}

/// Performs a bitwise OR operation on two `Number` values.
///
/// # Arguments
///
/// * `i1` - The first `Number` value.
/// * `i2` - The second `Number` value.
///
/// # Returns
///
/// Returns the result of performing a bitwise OR operation on `i1` and `i2`.
///
/// # Panics
///
/// Panics if either `i1` or `i2` is not an integer.
pub fn std_bitwise_or(i1: Number, i2: Number) -> Number {
	match (i1, i2) {
		(Number::Integer(val1), Number::Integer(val2)) => Number::Integer(val1 | val2),
		_ => panic!("Both inputs must be integers: {} and {}", i1, i2),
	}
}

/// Performs a bitwise XOR operation between two numbers.
///
/// # Arguments
///
/// * `i1` - The first input number.
/// * `i2` - The second input number.
///
/// # Panics
///
/// This function will panic if either `i1` or `i2` is not an integer.
///
/// # Returns
///
/// The result of the bitwise XOR operation between `i1` and `i2`.
pub fn std_bitwise_xor(i1: Number, i2: Number) -> Number {
	match (i1, i2) {
		(Number::Integer(val1), Number::Integer(val2)) => Number::Integer(val1 ^ val2),
		_ => panic!("Both inputs must be integers: {} and {}", i1, i2),
	}
}

/// Performs bitwise NOT operation on the given input, returning a new Number.
///
/// # Arguments
///
/// * `i` - The input Number to perform the operation on.
///
/// # Panics
///
/// Panics if the input is not an integer Number.
pub fn std_bitwise_not(i: Number) -> Number {
	match i {
		Number::Integer(val) => Number::Integer(!val),
		_ => panic!("Input must be an integer: {}", i),
	}
}