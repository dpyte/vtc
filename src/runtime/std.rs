use std::collections::HashMap;
use std::rc::Rc;

use crate::{Number, Value};

pub type VtcFn = Box<dyn Fn(Vec<Rc<Value>>) -> Rc<Value>>;

/// StdLibLoader
/// Load and manage standard library functions
pub struct StdLibLoader {
	loadable: HashMap<String, VtcFn>,
}


impl StdLibLoader {
	pub fn new() -> Self {
		let mut loadable = HashMap::new();

		// Helper macro to reduce boilerplate when adding functions
		macro_rules! add_fn {
            ($name:expr, $func:expr) => {
                loadable.insert(
                    $name.to_string(),
                    Box::new(move |args: Vec<Rc<Value>>| -> Rc<Value> {
                        if args.len() != 2 {
                            panic!("{} expects 2 arguments", $name);
                        }
                        let a = &args[0];
                        let b = &args[1];
                        if let (Value::Number(n1), Value::Number(n2)) = (a.as_ref(), b.as_ref()) {
                            Rc::new(Value::Number($func(n1.clone(), n2.clone())))
                        } else {
                            panic!("{} expects two Number values", $name);
                        }
                    }) as VtcFn
                );
            };
        }

		// Add integer operations
		add_fn!("std_add_int", std_add_int);
		add_fn!("std_sub_int", std_sub_int);
		add_fn!("std_mul_int", std_mul_int);
		add_fn!("std_div_int", std_div_int);
		add_fn!("std_mod_int", std_mod_int);

		// Add float operations
		add_fn!("std_add_float", std_add_float);
		add_fn!("std_sub_float", std_sub_float);
		add_fn!("std_mul_float", std_mul_float);
		add_fn!("std_div_float", std_div_float);

		// Add conversion functions
		loadable.insert(
			"std_int_to_float".to_string(),
			Box::new(|args: Vec<Rc<Value>>| -> Rc<Value> {
				if args.len() != 1 {
					panic!("std_int_to_float expects 1 argument");
				}
				if let Value::Number(n) = args[0].as_ref() {
					Rc::new(Value::Number(std_int_to_float(n.clone())))
				} else {
					panic!("std_int_to_float expects a Number value");
				}
			}) as VtcFn
		);

		loadable.insert(
			"std_float_to_int".to_string(),
			Box::new(|args: Vec<Rc<Value>>| -> Rc<Value> {
				if args.len() != 1 {
					panic!("std_float_to_int expects 1 argument");
				}
				if let Value::Number(n) = args[0].as_ref() {
					Rc::new(Value::Number(std_float_to_int(n.clone())))
				} else {
					panic!("std_float_to_int expects a Number value");
				}
			}) as VtcFn
		);

		// Add comparison functions
		loadable.insert(
			"std_eq".to_string(),
			Box::new(|args: Vec<Rc<Value>>| -> Rc<Value> {
				if args.len() != 2 {
					panic!("std_eq expects 2 arguments");
				}
				let a = &args[0];
				let b = &args[1];
				if let (Value::Number(n1), Value::Number(n2)) = (a.as_ref(), b.as_ref()) {
					Rc::new(Value::Boolean(std_eq(n1.clone(), n2.clone())))
				} else {
					panic!("std_eq expects two Number values");
				}
			}) as VtcFn
		);

		// Similarly, add std_lt and std_gt

		// Add bitwise operations
		add_fn!("std_bitwise_and", std_bitwise_and);
		add_fn!("std_bitwise_or", std_bitwise_or);
		add_fn!("std_bitwise_xor", std_bitwise_xor);

		loadable.insert(
			"std_bitwise_not".to_string(),
			Box::new(|args: Vec<Rc<Value>>| -> Rc<Value> {
				if args.len() != 1 {
					panic!("std_bitwise_not expects 1 argument");
				}
				if let Value::Number(n) = args[0].as_ref() {
					Rc::new(Value::Number(std_bitwise_not(n.clone())))
				} else {
					panic!("std_bitwise_not expects a Number value");
				}
			}) as VtcFn
		);

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