#[cfg(test)]
mod tests {
    use vtc::runtime::Runtime;

    #[test]
    fn test_std_if_intrinsic() {
        // Create a new Runtime instance
        let mut runtime = Runtime::new();

        // Define a VTC configuration with if-else tests
        let vtc_config = r#"
        @test_if_else:
            $true_condition := [std_if!!, True, "condition is true", "condition is false"]
            $false_condition := [std_if!!, False, "condition is true", "condition is false"]
            $numeric_condition := [std_if!!, [std_lt!!, 5, 10], "5 is less than 10", "5 is not less than 10"]
        "#;

        // Load the configuration into the runtime
        runtime
            .load_vtc(vtc_config)
            .expect("Failed to load VTC configuration");

        // Test true condition
        let true_result = runtime
            .get_string("test_if_else", "true_condition")
            .expect("Failed to get true_condition value");
        assert_eq!(true_result, "condition is true");

        // Test false condition
        let false_result = runtime
            .get_string("test_if_else", "false_condition")
            .expect("Failed to get false_condition value");
        assert_eq!(false_result, "condition is false");

        // Test numeric condition
        let numeric_result = runtime
            .get_string("test_if_else", "numeric_condition")
            .expect("Failed to get numeric_condition value");
        assert_eq!(numeric_result, "5 is less than 10");
    }
}
