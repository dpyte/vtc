#[cfg(test)]
mod tests {
	use std::rc::Rc;
	use vtc::runtime::Runtime;
	use vtc::runtime::std::{StdLibLoader, VtcFn};
	use vtc::value::{Number, Value};

	// Helper function to create a runtime with some predefined values
	fn setup_runtime() -> Runtime {
		let mut rt = Runtime::new();
		rt.load_vtc(r#"
            @test_namespace:
                # Single line comment
                $int_value := 42
                $float_value := [multiply_and_concat!!, 2, 2]
        "#).unwrap();
		rt
	}

	fn multiply_and_concatenate(args: Vec<Rc<Value>>) -> Rc<Value> {
		let result = match &*args[0] {
			Value::Number(Number::Integer(i)) => Rc::new(Value::Number(Number::Integer(i * 2))),
			Value::Number(Number::Float(f)) => Rc::new(Value::Number(Number::Float(f * 2.0))),
			_ => panic!("multiply_and_concatenate_expects a number")
		};

		let mut customer_string = String::from("testing_");
		customer_string.push(result.to_string().parse().unwrap());
		Rc::new(Value::String(Rc::from(customer_string)))
	}

	#[test]
	fn test_comments() {
		let mut rt = setup_runtime();
		let mut libloader = StdLibLoader::new();
		libloader.register_function(
			"multiply_and_concat".to_string(), Box::new(multiply_and_concatenate) as VtcFn).unwrap();
		rt.update_library_loader(libloader).unwrap();

		let value = rt.get_string("test_namespace", "float_value").unwrap();
		assert_eq!(value, "testing_4");
	}
}
