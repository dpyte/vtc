use crate::value::*;

pub(crate) mod lists {
	use super::*;

	pub fn flatten_value(value: &Value) -> Vec<Value> {
		let mut result = Vec::new();
		flatten_value_recursively(value, &mut result);
		result
	}

	fn flatten_value_recursively(value: &Value, result: &mut Vec<Value>) {
		match value {
			Value::List(list) => {
				for item in list.iter() {
					flatten_value_recursively(item, result);
				}
			}
			_ => result.push(value.clone()),
		}
	}
}
