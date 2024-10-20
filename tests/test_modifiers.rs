#[cfg(test)]
mod tests {
	use std::rc::Rc;
	use vtc::runtime::Runtime;
	use vtc::value::{Number, Value};

	#[test]
	fn test_add_value_to_existing_namespace() {
		let mut runtime = Runtime::new();
		runtime.add_namespace("test_ns").unwrap();
		assert!(runtime.add_value("test_ns", "key1", Value::String(
			Rc::new(String::from("value1")))).is_ok());
		// Assert that the value was added correctly
	}

	#[test]
	fn test_add_value_to_non_existent_namespace() {
		let mut runtime = Runtime::new();
		assert!(runtime.add_value("new_ns", "key1", Value::String(
			Rc::new(String::from("value1")))).is_ok());
		// Assert that the namespace was created and the value was added
	}

	#[test]
	fn test_add_different_value_types() {
		let mut runtime = Runtime::new();
		assert!(runtime.add_value("ns", "string_key", Value::String(
			Rc::new(String::from("value1")))).is_ok());
		assert!(runtime.add_value("ns", "int_key", Value::Number(Number::Integer(42))).is_ok());
		assert!(runtime.add_value("ns", "bool_key", Value::Boolean(true)).is_ok());
	}

	#[test]
	fn test_update_existing_value() {
		let mut runtime = Runtime::new();
		runtime.add_value("ns", "key", Value::String(
			Rc::new(String::from("")))).unwrap();
		assert!(runtime.add_value("ns", "key", Value::String(
			Rc::new(String::from("new")))).is_ok());
	}

	#[test]
	fn test_remove_existing_value() {
		let mut runtime = Runtime::new();
		runtime.add_value("ns", "key", Value::String(
			Rc::new(String::from("value")))).unwrap();
		assert!(runtime.delete_value("ns", "key").is_ok());
		// Assert that the value was removed
	}

	#[test]
	fn test_remove_non_existent_value() {
		let mut runtime = Runtime::new();
		assert!(runtime.delete_value("ns", "non_existent_key").is_err());
	}
}