use std::fs;
use std::path::PathBuf;
use tempfile::tempdir;

#[cfg(test)]
mod tests {
	use std::sync::Arc;
	use fnv::FnvHashMap;
	use vtc::runtime::Runtime;
	use vtc::value::{Accessor, Number, Reference, ReferenceType, Value};
	use super::*;

	fn setup() -> (Runtime, PathBuf) {
		let runtime = Runtime::new();
		let temp_dir = tempdir().expect("Failed to create temp directory");
		let output_path = temp_dir.path().join("test_output.vtc");
		(runtime, output_path)
	}

	#[test]
	fn test_empty_runtime() {
		let (runtime, output_path) = setup();
		runtime.dump_to_file(&output_path).unwrap();
		let content = fs::read_to_string(&output_path).unwrap();
		assert!(content.is_empty(), "File should be empty for an empty runtime");
	}

	#[test]
	fn test_single_namespace_single_variable() {
		let (mut runtime, output_path) = setup();

		runtime.namespaces.insert(
			Arc::new("test_namespace".to_string()),
			{
				let mut map = FnvHashMap::default();
				map.insert(
					Arc::new("test_var".to_string()),
					Arc::new(Value::String("Hello, World!".to_string()))
				);
				map
			}
		);

		runtime.dump_to_file(&output_path).unwrap();
		let content = fs::read_to_string(&output_path).unwrap();

		// Using line-by-line comparison for better error messages
		let lines: Vec<&str> = content.lines().collect();
		assert_eq!(lines[0], "@test_namespace:");
		assert_eq!(lines[1], "\t$test_var := \"Hello, World!\"");
		assert!(lines[2].is_empty());
	}

	#[test]
	fn test_multiple_namespaces() {
		let (mut runtime, output_path) = setup();

		// First namespace with multiple variables
		runtime.namespaces.insert(
			Arc::new("namespace1".to_string()),
			{
				let mut map = FnvHashMap::default();
				map.insert(Arc::new("var1".to_string()), Arc::new(Value::Number(Number::Integer(42))));
				map.insert(Arc::new("var2".to_string()), Arc::new(Value::Boolean(true)));
				map
			}
		);

		// Second namespace with a single variable
		runtime.namespaces.insert(
			Arc::new("namespace2".to_string()),
			{
				let mut map = FnvHashMap::default();
				map.insert(Arc::new("var3".to_string()), Arc::new(Value::Number(Number::Float(3.14))));
				map
			}
		);

		runtime.dump_to_file(&output_path).unwrap();
		let content = fs::read_to_string(&output_path).unwrap();

		// Split content into namespace blocks and filter out empty strings
		let blocks: Vec<&str> = content.split("\n\n")
			.filter(|block| !block.trim().is_empty())
			.collect();

		// Verify we have exactly 2 namespace blocks
		assert_eq!(blocks.len(), 2, "Expected 2 namespace blocks, found {}", blocks.len());

		// Create a map of namespace content for easier verification
		let mut namespace_contents: std::collections::HashMap<&str, Vec<&str>> = std::collections::HashMap::new();

		for block in blocks {
			let lines: Vec<&str> = block.lines().collect();
			if lines.is_empty() { continue; }

			// Get namespace name without the '@' and ':'
			let namespace = lines[0].trim_start_matches('@').trim_end_matches(':');
			namespace_contents.insert(namespace, lines[1..].to_vec());
		}

		// Verify namespace1
		if let Some(var_lines) = namespace_contents.get("namespace1") {
			let has_var1 = var_lines.iter().any(|line| {
				line.trim() == "$var1 := 42"
			});
			let has_var2 = var_lines.iter().any(|line| {
				line.trim() == "$var2 := True"
			});

			assert!(has_var1, "Missing var1 in namespace1");
			assert!(has_var2, "Missing var2 in namespace1");
		} else {
			panic!("namespace1 not found in output");
		}

		// Verify namespace2
		if let Some(var_lines) = namespace_contents.get("namespace2") {
			let has_var3 = var_lines.iter().any(|line| {
				line.trim() == "$var3 := 3.14"
			});

			assert!(has_var3, "Missing var3 in namespace2");
		} else {
			panic!("namespace2 not found in output");
		}
	}

	#[test]
	fn test_list_values() {
		let (mut runtime, output_path) = setup();

		runtime.namespaces.insert(
			Arc::new("test_namespace".to_string()),
			{
				let mut map = FnvHashMap::default();
				map.insert(
					Arc::new("list_var".to_string()),
					Arc::new(Value::List(Arc::new(vec![
						Value::Number(Number::Integer(1)),
						Value::String("two".to_string()),
						Value::Boolean(true)
					])))
				);
				map
			}
		);

		runtime.dump_to_file(&output_path).unwrap();
		let content = fs::read_to_string(&output_path).unwrap();
		let lines: Vec<&str> = content.lines().collect();

		assert_eq!(lines[0], "@test_namespace:");
		assert!(lines[1].trim().starts_with("$list_var := ["));
		assert!(lines[1].contains("1"));
		assert!(lines[1].contains("\"two\""));
		assert!(lines[1].contains("True"));
		assert!(lines[1].ends_with("]"));
	}

	#[test]
	fn test_reference_values() {
		let (mut runtime, output_path) = setup();

		runtime.namespaces.insert(
			Arc::new("test_namespace".to_string()),
			{
				let mut map = FnvHashMap::default();
				map.insert(
					Arc::new("ref_var".to_string()),
					Arc::new(Value::Reference(Reference {
						ref_type: ReferenceType::Local,
						namespace: Some(Arc::new("other_namespace".to_string())),
						variable: Arc::new("other_var".to_string()),
						accessors: smallvec::smallvec![
                            Accessor::Index(0),
                            Accessor::Range(1, 3),
                            Accessor::Key("key".to_string())
                        ]
					}))
				);
				map
			}
		);

		runtime.dump_to_file(&output_path).unwrap();
		let content = fs::read_to_string(&output_path).unwrap();
		let lines: Vec<&str> = content.lines().collect();

		assert_eq!(lines[0], "@test_namespace:");
		let ref_line = lines[1].trim();
		assert!(ref_line.starts_with("$ref_var := %"));
		assert!(ref_line.contains("other_namespace.other_var"));
		assert!(ref_line.contains("->(0)"));
		assert!(ref_line.contains("->(1, 3)"));
		assert!(ref_line.contains("->[key]"));
	}

	#[test]
	fn test_intrinsic_values() {
		let (mut runtime, output_path) = setup();

		runtime.namespaces.insert(
			Arc::new("test_namespace".to_string()),
			{
				let mut map = FnvHashMap::default();
				map.insert(
					Arc::new("intrinsic_var".to_string()),
					Arc::new(Value::Intrinsic("std_add_int".to_string()))
				);
				map
			}
		);

		runtime.dump_to_file(&output_path).unwrap();
		let content = fs::read_to_string(&output_path).unwrap();
		let lines: Vec<&str> = content.lines().collect();

		assert_eq!(lines[0], "@test_namespace:");
		assert!(lines[1].trim().starts_with("$intrinsic_var :="));
		assert!(lines[1].trim().ends_with("std_add_int!!"));
	}

	#[test]
	fn test_value_serialization() {
		let runtime = Runtime::new();

		// Boolean serialization
		assert_eq!(runtime.serialize_value(&Value::Boolean(true)), "True");
		assert_eq!(runtime.serialize_value(&Value::Boolean(false)), "False");

		// String serialization
		assert_eq!(runtime.serialize_value(&Value::String("hello".to_string())), "\"hello\"");
		assert_eq!(runtime.serialize_value(&Value::String("hello \"world\"".to_string())), "hello \"world\"");

		// Number serialization
		assert_eq!(runtime.serialize_value(&Value::Number(Number::Integer(42))), "42");
		assert_eq!(runtime.serialize_value(&Value::Number(Number::Float(3.14))), "3.14");
		assert_eq!(runtime.serialize_value(&Value::Number(Number::Binary(0b1010))), "0b1010");
		assert_eq!(runtime.serialize_value(&Value::Number(Number::Hexadecimal(0xFF))), "0xFF");
	}
}