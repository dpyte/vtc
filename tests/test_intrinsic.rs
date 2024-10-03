#[cfg(test)]
mod tests {
	use std::collections::HashSet;
	use std::rc::Rc;
	use vtc::runtime::runtime::Runtime;
	use vtc::{Number, Value};
	use vtc::runtime::error::RuntimeError;

	// Helper function to create a runtime with some predefined values
	fn setup_runtime() -> Runtime {
		let mut rt = Runtime::new();
		rt.load_vtc(r#"
            @test_namespace:
                $int_value := 42
                $float_value := 3.14
                $string_value := "Hello, World!"
                $list_value := [1, 2, 3, 4, 5]
                $nested_list := [[1, 2], [3, 4], [5, 6]]
                $add_ints := [std_add_int!!, 10, 20]
                $add_floats := [std_add_float!!, 1.5, 2.5]
                $nested_intrinsic := [std_mul_int!!, [std_add_int!!, 5, 5], 2]
        "#).unwrap();
		rt
	}

	#[test]
	fn test_add_int_intrinsic() {
		let rt = setup_runtime();
		let result = rt.get_value("test_namespace", "add_ints", &[]).unwrap();
		assert_eq!(*result, Value::Number(Number::Integer(30)));
	}

	#[test]
	fn test_add_float_intrinsic() {
		let rt = setup_runtime();
		let result = rt.get_value("test_namespace", "add_floats", &[]).unwrap();
		assert_eq!(*result, Value::Number(Number::Float(4.0)));
	}

	#[test]
	fn test_nested_intrinsic() {
		let rt = setup_runtime();
		let result = rt.get_value("test_namespace", "nested_intrinsic", &[]).unwrap();
		assert_eq!(*result, Value::Number(Number::Integer(20)));
	}

	#[test]
	fn test_intrinsic_with_variable_args() {
		let rt = setup_runtime();
		let result = rt.get_value("test_namespace", "add_ints", &[]).unwrap();
		let new_intrinsic = Rc::new(Value::List(Rc::new(vec![
			Rc::new(Value::Intrinsic(Rc::new("std_add_int".to_string()))),
			result,
			Rc::new(Value::Number(Number::Integer(5))),
		])));
		let final_result = rt.resolve_intrinsics(new_intrinsic, &mut HashSet::new()).unwrap();
		assert_eq!(*final_result, Value::Number(Number::Integer(35)));
	}

	#[test]
	fn test_unknown_intrinsic() {
		let rt = setup_runtime();
		let unknown_intrinsic = Rc::new(Value::Intrinsic(Rc::new("unknown_function".to_string())));
		let result = rt.resolve_intrinsics(unknown_intrinsic, &mut HashSet::new());
		assert!(matches!(result, Err(RuntimeError::UnknownIntrinsic(_))));
	}

	#[test]
	fn test_intrinsic_with_invalid_args() {
		let rt = setup_runtime();
		let invalid_intrinsic = Rc::new(Value::Intrinsic(Rc::new("std_add_int".to_string())));
		let result = rt.resolve_intrinsics(invalid_intrinsic, &mut HashSet::new());
		assert!(matches!(result, Err(RuntimeError::InvalidIntrinsicArgs)));
	}

	#[test]
	fn test_resolve_list_of_intrinsics() {
		let rt = setup_runtime();
		let list_of_intrinsics = Rc::new(Value::List(Rc::new(vec![
			Rc::new(Value::List(Rc::new(vec![
				Rc::new(Value::Intrinsic(Rc::new("std_add_int".to_string()))),
				Rc::new(Value::Number(Number::Integer(5))),
				Rc::new(Value::Number(Number::Integer(10))),
			]))),
			Rc::new(Value::List(Rc::new(vec![
				Rc::new(Value::Intrinsic(Rc::new("std_mul_int".to_string()))),
				Rc::new(Value::Number(Number::Integer(3))),
				Rc::new(Value::Number(Number::Integer(7))),
			]))),
		])));
		let result = rt.resolve_intrinsics(list_of_intrinsics, &mut HashSet::new()).unwrap();
		if let Value::List(resolved_list) = &*result {
			assert_eq!(resolved_list.len(), 2);
			assert_eq!(*resolved_list[0], Value::Number(Number::Integer(15)));
			assert_eq!(*resolved_list[1], Value::Number(Number::Integer(21)));
		} else {
			panic!("Expected a list result");
		}
	}

	#[test]
	fn test_intrinsic_type_mismatch() {
		let rt = setup_runtime();
		let mismatched_intrinsic = Rc::new(Value::List(Rc::new(vec![
			Rc::new(Value::Intrinsic(Rc::new("std_add_int".to_string()))),
			Rc::new(Value::Number(Number::Integer(5))),
			Rc::new(Value::Number(Number::Float(10.5))),
		])));
		let result = rt.resolve_intrinsics(mismatched_intrinsic, &mut HashSet::new());
		assert!(matches!(result, Err(RuntimeError::TypeError(_))));
	}
}