#[cfg(test)]
mod tests {
	use vtc::runtime::runtime::Runtime;

	// Helper function to create a runtime with some predefined values
	fn setup_runtime() -> Runtime {
		let mut rt = Runtime::new();
		rt.load_vtc(r#"
            @test_namespace:
                # Single line comment
                $int_value := 42
                $float_value := 3.14
        "#).unwrap();
		rt
	}

	#[test]
	fn test_comments() {
		let rt = setup_runtime();
		let value = rt.get_float("test_namespace", "float_value").unwrap();
		assert_eq!(value, 3.14);
	}
}
