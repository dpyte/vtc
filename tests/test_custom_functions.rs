#[cfg(test)]
mod tests {
	use std::sync::Arc;
	use vtc::runtime::Runtime;
	use vtc::runtime::std::{StdLibLoader, VtcFn};
	use vtc::value::{Number, Value};

	fn multiply_and_concatenate(args: Vec<Arc<Value>>) -> Arc<Value> {
		if args.len() != 2 {
			panic!("multiply_and_concat expects 2 arguments");
		}

		let num1 = match &*args[0] {
			Value::Number(Number::Integer(i)) => *i as f64,
			Value::Number(Number::Float(f)) => *f,
			_ => panic!("multiply_and_concat expects numbers")
		};

		let num2 = match &*args[1] {
			Value::Number(Number::Integer(i)) => *i as f64,
			Value::Number(Number::Float(f)) => *f,
			_ => panic!("multiply_and_concat expects numbers")
		};

		let result = num1 * num2;
		let result_string = format!("testing_{}", result);
		Arc::new(Value::String(result_string))
	}

	fn setup_runtime() -> Runtime {
		let mut libloader = StdLibLoader::new();
		// Add the !! suffix to match the VTC syntax
		libloader.register_function(
			"multiply_and_concat".to_string(),
			Box::new(multiply_and_concatenate) as VtcFn
		).unwrap();

		let mut rt = Runtime::new();
		rt.update_library_loader(libloader).unwrap();

		rt.load_vtc(r#"
            @test_namespace:
                # Single line comment
                $int_value := 42
                $float_value := [multiply_and_concat!!, 2, 2]
        "#).unwrap();

		rt
	}

	#[test]
	fn test_comments() {
		let rt = setup_runtime();
		let value = rt.get_string("test_namespace", "float_value").unwrap();
		assert_eq!(value, "testing_4");
	}
}