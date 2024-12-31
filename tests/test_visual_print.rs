#[cfg(test)]
mod tests {
	use vtc::runtime::Runtime;

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
	fn test_visual_debug_print() {
		let rt = setup_runtime();
		println!("{:?}", rt);
		let result = rt.get_integer("test_namespace", "add_ints").unwrap();
		assert_eq!(result, 30);
	}
}