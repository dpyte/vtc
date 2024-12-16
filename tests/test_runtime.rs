#[cfg(test)]
mod tests {
	use std::sync::Arc;

	use fnv::FnvHashMap;

	use vtc::runtime::Runtime;
	use vtc::value::Number::Integer;
	use vtc::value::Value;
	use vtc::value::Value::Number;

	// Helper function to create a runtime with some predefined values
	fn setup_runtime() -> Runtime {
		let mut rt = Runtime::new();
		rt.load_vtc(r#"
            @test_namespace:
                $int_value := 42
                $float_value := 3.14
                $string_value := "Hello, World!"
                $list_value := [1, 2, 3, 4, 5]
                $list_to_dict := [1, 2, 3, 4]
                $nested_list := [[1, 2], [3, 4], [5, 6], [7, [8, [9]]]]
                $add_ints := [std_add_int!!, 10, 20]
                $add_floats := [std_add_float!!, 1.5, 2.5]
                $nested_intrinsic := [std_mul_int!!, [std_add_int!!, 5, 5], 2]
                $complex_dict := [
                    %test_namespace.string_value,
                    [std_add_int!!, 12, 12]
                ]
        "#).unwrap();
		rt
	}

	#[test]
	fn test_as_dict() {
		let rt = setup_runtime();
		let list = rt.as_dict("test_namespace", "complex_dict").unwrap();

		let mut match_against = FnvHashMap::default();
		match_against.insert(String::from(r#"Hello, World!"#), Arc::new(Number(Integer(24))));
		println!("{:?}", match_against);
		assert_eq!(list, match_against);
	}

	#[test]
	fn test_flatten_list() {
		let rt = setup_runtime();
		let list: Vec<Value> = rt.flatten_list("test_namespace", "nested_list").unwrap();

		let match_against: Vec<Value> = vec![
			Number(Integer(1)),
			Number(Integer(2)),
			Number(Integer(3)),
			Number(Integer(4)),
			Number(Integer(5)),
			Number(Integer(6)),
			Number(Integer(7)),
			Number(Integer(8)),
			Number(Integer(9)),
		];

		println!("{:?}", list);
		assert_eq!(list, match_against);
	}
}