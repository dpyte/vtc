use std::collections::HashMap;
use std::rc::Rc;
use crate::runtime::error::RuntimeError;
use crate::runtime::{runtime, Runtime};
use crate::value::{Number, Value};

impl Runtime {
	pub fn get_string_value(&self, value: &Rc<Value>) -> Result<String, RuntimeError> {
		if let Value::String(s) = &**value {
			Ok((**s).clone())
		} else {
			Err(RuntimeError::TypeError("Expected string".to_string()))
		}
	}

	pub fn get_string(&self, namespace: &str, variable: &str) -> Result<String, RuntimeError> {
		let value = self.get_value(namespace, variable, &[])?;
		self.get_string_value(&value)
	}

	pub fn to_string(&self, value: Rc<Value>) -> Result<String, RuntimeError> {
		Ok(value.to_string()[1..value.to_string().len() - 1].to_string())
	}

	pub fn get_integer(&self, namespace: &str, variable: &str) -> Result<i64, RuntimeError> {
		self.get_typed_value(namespace, variable, |v| {
			if let Value::Number(Number::Integer(i)) = &**v {
				Ok(*i)
			} else {
				Err(RuntimeError::TypeError("Expected integer".to_string()))
			}
		})
	}

	pub fn get_float(&self, namespace: &str, variable: &str) -> Result<f64, RuntimeError> {
		self.get_typed_value(namespace, variable, |v| {
			if let Value::Number(Number::Float(f)) = &**v {
				Ok(*f)
			} else {
				Err(RuntimeError::TypeError("Expected float".to_string()))
			}
		})
	}

	pub fn get_boolean(&self, namespace: &str, variable: &str) -> Result<bool, RuntimeError> {
		self.get_typed_value(namespace, variable, |v| {
			if let Value::Boolean(b) = &**v {
				Ok(*b)
			} else {
				Err(RuntimeError::TypeError("Expected boolean".to_string()))
			}
		})
	}

	pub fn get_list(&self, namespace: &str, variable: &str) -> Result<Vec<Value>, RuntimeError> {
		self.get_typed_value(namespace, variable, |v| {
			if let Value::List(l) = &**v {
				Ok(l
					.iter()
					.map(|value| (**value).clone())
					.collect::<Vec<_>>())
			} else {
				Err(RuntimeError::TypeError("Expected list".to_string()))
			}
		})
	}

	/// Convert list to dictionary.
	/// Constraints: The size of list must be in multiples of 2.

	pub fn as_dict(&self, namespace: &str, variable: &str) -> Result<HashMap<String, Rc<Value>>, RuntimeError> {
		let values = self.get_list(namespace, variable)?;
		let mut result = HashMap::new();

		for value in values {
			match value.as_ref() {
				Value::List(inner_list) => {
					if inner_list.len() != 2 {
						return Err(RuntimeError::ConversionError(
							format!("Invalid key-value pair in {}::{}", namespace, variable)
						));
					}
					let key = match inner_list[0].as_ref() {
						Value::String(s) => s.as_ref().clone(),
						_ => return Err(RuntimeError::ConversionError(
							format!("Key must be a string in {}::{}", namespace, variable)
						)),
					};
					result.insert(key, Rc::clone(&inner_list[1]));
				},
				_ => return Err(RuntimeError::ConversionError(
					format!("Expected list of key-value pairs in {}::{}", namespace, variable)
				)),
			}
		}

		Ok(result)
	}


	/// flattens a nested list to a single dimension list
	pub fn flatten_list(&self, namespace: &str, variable: &str) -> Result<Vec<Value>, RuntimeError> {
		let list = self.get_list(namespace, variable)?;
		let rc_list = list.into_iter().map(|value| Rc::new(value)).collect::<Vec<_>>();
		Ok(runtime::lists::flatten_value(&Value::List(Rc::new(rc_list))))
	}
}
