#[cfg(test)]
mod tests {
	use std::path::PathBuf;
	use std::sync::Arc;
	use std::vec;
	use fnv::FnvHashSet;

	use vtc::runtime::error::RuntimeError;
	use vtc::runtime::Runtime;
	use vtc::value::{Number, Value};

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
		let result = rt.get_integer("test_namespace", "add_ints").unwrap();
		assert_eq!(result, 30);
	}

	#[test]
	fn test_add_float_intrinsic() {
		let rt = setup_runtime();
		let result = rt.get_float("test_namespace", "add_floats").unwrap();
		assert_eq!(result, 4.0);
	}

	#[test]
	fn test_nested_intrinsic() {
		let rt = setup_runtime();
		let result = rt.get_integer("test_namespace", "nested_intrinsic").unwrap();
		assert_eq!(result, 20);
	}

	#[test]
	fn test_intrinsic_with_variable_args() {
		let rt = setup_runtime();
		let result = rt.get_value("test_namespace", "add_ints", &[]).unwrap();
		let new_intrinsic = Arc::new(Value::List(Arc::new(vec![
			Value::Intrinsic("std_add_int".to_string()),
			(*result).clone(),
			Value::Number(Number::Integer(5)),
		])));
		let final_result = rt.resolve_intrinsics(new_intrinsic, &mut FnvHashSet::default()).unwrap();
		assert_eq!(*final_result, Value::Number(Number::Integer(35)));
	}

	#[test]
	fn test_unknown_intrinsic() {
		let rt = setup_runtime();
		let unknown_intrinsic = Value::List(Arc::new(vec![Value::Intrinsic("unknown_function".to_string())]));
		let result = rt.resolve_intrinsics(Arc::from(unknown_intrinsic), &mut FnvHashSet::default());
		assert!(matches!(result, Err(RuntimeError::UnknownIntrinsic(_))));
	}

	#[test]
	fn test_intrinsic_with_invalid_args() {
		let rt = setup_runtime();
		let invalid_intrinsic = Arc::new(Value::Intrinsic("std_add_int".to_string()));
		let result = rt.resolve_intrinsics(invalid_intrinsic, &mut FnvHashSet::default());
		// assert!(matches!(result, Err(RuntimeError::InvalidIntrinsicArgs)));
	}

	#[test]
	fn test_resolve_list_of_intrinsics() {
		let rt = setup_runtime();
		let list_of_intrinsics = Arc::new(Value::List(Arc::new(vec![
			Value::List(Arc::new(vec![
				Value::Intrinsic("std_add_int".to_string()),
				Value::Number(Number::Integer(5)),
				Value::Number(Number::Integer(10)),
			])),
			Value::List(Arc::new(vec![
				Value::Intrinsic("std_mul_int".to_string()),
				Value::Number(Number::Integer(3)),
				Value::Number(Number::Integer(7)),
			])),
		])));
		let result = rt.resolve_intrinsics(list_of_intrinsics, &mut FnvHashSet::default()).unwrap();
		if let Value::List(resolved_list) = &*result {
			assert_eq!(resolved_list.len(), 2);
			assert_eq!(resolved_list[0], Value::Number(Number::Integer(15)));
			assert_eq!(resolved_list[1], Value::Number(Number::Integer(21)));
		} else {
			panic!("Expected a list result");
		}
	}

	#[test]
	fn test_intrinsic_type_mismatch() {
		let rt = setup_runtime();
		let mismatched_intrinsic = Arc::new(Value::List(Arc::new(vec![
			Value::Intrinsic("std_add_int".to_string()),
			Value::Number(Number::Integer(5)),
			Value::Number(Number::Float(10.5)),
		])));
		let result = rt.resolve_intrinsics(mismatched_intrinsic, &mut FnvHashSet::default());
		// assert!(matches!(result, Err(RuntimeError::TypeError(_))));
	}

	#[test]
	fn test_large_intrinsic() {
		let rt = Runtime::from(PathBuf::from("./samples/large_config.vtc")).unwrap();
		let result = rt.get_value("string_operations", "lowercase", &[]).unwrap();
		println!(">> {:?}", result);
		let _res = result;
	}

	#[test]
	fn test_gt_intrinsic() {
		let rt = Runtime::from(PathBuf::from("./samples/large_config.vtc")).unwrap();
		let result = rt.get_value("conditional_logic", "condition_1", &[]).unwrap();
		println!(">> {:?}", result);
		assert_eq!(*result, Value::Boolean(true));
	}

	#[test]
	fn test_lt_intrinsic() {
		let rt = Runtime::from(PathBuf::from("./samples/large_config.vtc")).unwrap();
		let result = rt.get_value("conditional_logic", "condition_2", &[]).unwrap();
		println!(">> {:?}", result);
		assert_eq!(*result, Value::Boolean(true));
	}
}