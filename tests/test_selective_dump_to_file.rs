#[cfg(test)]
mod tests {
	use std::collections::HashMap;
	use std::fs;
	use std::path::PathBuf;
	use std::rc::Rc;

	use vtc::runtime::Runtime;
	use vtc::value::{Number, Reference, ReferenceType, Value};

	fn setup() -> (Runtime, PathBuf) {
		let runtime = Runtime::new();
		let output_path = PathBuf::from("test_selective_output.vtc");
		fs::write(&output_path, "").unwrap(); // Create an empty file
		(runtime, output_path)
	}

	fn teardown(path: &PathBuf) {
		fs::remove_file(path).unwrap_or_default();
	}

	#[test]
	fn test_selective_dump_empty_runtime() {
		let (runtime, output_path) = setup();
		runtime.dump_selective(&output_path, vec![]).unwrap();
		let content = fs::read_to_string(&output_path).unwrap();
		assert!(content.is_empty(), "File should be empty for an empty runtime");
		teardown(&output_path);
	}

	#[test]
	fn test_selective_dump_single_namespace() {
		let (mut runtime, output_path) = setup();

		runtime.namespaces.insert(
			Rc::new("test_namespace".to_string()),
			{
				let mut map = HashMap::new();
				map.insert(
					Rc::new("test_var".to_string()),
					Rc::new(Value::String(Rc::new("Hello, World!".to_string())))
				);
				map
			}
		);

		runtime.dump_selective(&output_path, vec!["test_namespace".to_string()]).unwrap();

		let content = fs::read_to_string(&output_path).unwrap();
		println!("Actual content:\n{}", content);
		assert_eq!(content, "@test_namespace:\n    $test_var := \"Hello, World!\"\n\n");

		teardown(&output_path);
	}

	#[test]
	fn test_selective_dump_multiple_namespaces() {
		let (mut runtime, output_path) = setup();

		runtime.namespaces.insert(
			Rc::new("namespace1".to_string()),
			{
				let mut map = HashMap::new();
				map.insert(Rc::new("var1".to_string()), Rc::new(Value::Number(Number::Integer(42))));
				map
			}
		);

		runtime.namespaces.insert(
			Rc::new("namespace2".to_string()),
			{
				let mut map = HashMap::new();
				map.insert(Rc::new("var2".to_string()), Rc::new(Value::Number(Number::Float(3.14))));
				map
			}
		);

		runtime.dump_selective(&output_path, vec!["namespace1".to_string(), "namespace2".to_string()]).unwrap();

		let content = fs::read_to_string(&output_path).unwrap();
		println!("Actual content:\n{}", content);
		assert!(content.contains("@namespace1:"));
		assert!(content.contains("$var1 := 42"));
		assert!(content.contains("@namespace2:"));
		assert!(content.contains("$var2 := 3.14"));

		teardown(&output_path);
	}

	#[test]
	fn test_selective_dump_with_dependencies() {
		let (mut runtime, output_path) = setup();

		runtime.namespaces.insert(
			Rc::new("namespace1".to_string()),
			{
				let mut map = std::collections::HashMap::new();
				map.insert(Rc::new("var1".to_string()), Rc::new(Value::Number(Number::Integer(42))));
				map
			}
		);

		runtime.namespaces.insert(
			Rc::new("namespace2".to_string()),
			{
				let mut map = std::collections::HashMap::new();
				map.insert(
					Rc::new("var2".to_string()),
					Rc::new(Value::Reference(Rc::new(Reference {
						ref_type: ReferenceType::Local,
						namespace: Some(Rc::new("namespace1".to_string())),
						variable: Rc::new("var1".to_string()),
						accessors: smallvec::smallvec![]
					})))
				);
				map
			}
		);

		runtime.dump_selective(&output_path, vec!["namespace2".to_string()]).unwrap();

		let content = fs::read_to_string(&output_path).unwrap();
		assert!(content.contains("@namespace1:"));
		assert!(content.contains("$var1 := 42"));
		assert!(content.contains("@namespace2:"));
		assert!(content.contains("$var2 := %namespace1.var1"));

		teardown(&output_path);
	}

	#[test]
	fn test_selective_dump_with_intrinsics() {
		let (mut runtime, output_path) = setup();

		runtime.namespaces.insert(
			Rc::new("intrinsic_namespace".to_string()),
			{
				let mut map = std::collections::HashMap::new();
				map.insert(
					Rc::new("intrinsic_var".to_string()),
					Rc::new(Value::Intrinsic(Rc::from("std_add_int".to_string())))
				);
				map
			}
		);

		runtime.dump_selective(&output_path, vec!["intrinsic_namespace".to_string()]).unwrap();

		let content = fs::read_to_string(&output_path).unwrap();
		assert!(content.contains("@intrinsic_namespace:"));
		assert!(content.contains("$intrinsic_var := std_add_int!!"));

		teardown(&output_path);
	}

	#[test]
	fn test_selective_dump_nonexistent_namespace() {
		let (mut runtime, output_path) = setup();
		runtime.namespaces.insert(
			Rc::new("existing_namespace".to_string()),
			{
				let mut map = HashMap::new();
				map.insert(Rc::new("var".to_string()), Rc::new(Value::Number(Number::Integer(42))));
				map
			}
		);

		runtime.dump_selective(&output_path, vec!["nonexistent_namespace".to_string()]).unwrap();
		let content = fs::read_to_string(&output_path).unwrap();
		assert!(content.is_empty(), "File should be empty when dumping nonexistent namespace");
		teardown(&output_path);
	}

	#[test]
	fn test_selective_dump_circular_reference() {
		let (mut runtime, output_path) = setup();

		runtime.namespaces.insert(
			Rc::new("namespace1".to_string()),
			{
				let mut map = std::collections::HashMap::new();
				map.insert(
					Rc::new("var1".to_string()),
					Rc::new(Value::Reference(Rc::new(Reference {
						ref_type: ReferenceType::Local,
						namespace: Some(Rc::new("namespace2".to_string())),
						variable: Rc::new("var2".to_string()),
						accessors: smallvec::smallvec![]
					})))
				);
				map
			}
		);

		runtime.namespaces.insert(
			Rc::new("namespace2".to_string()),
			{
				let mut map = std::collections::HashMap::new();
				map.insert(
					Rc::new("var2".to_string()),
					Rc::new(Value::Reference(Rc::new(Reference {
						ref_type: ReferenceType::Local,
						namespace: Some(Rc::new("namespace1".to_string())),
						variable: Rc::new("var1".to_string()),
						accessors: smallvec::smallvec![]
					})))
				);
				map
			}
		);

		runtime.dump_selective(&output_path, vec!["namespace1".to_string()]).unwrap();

		let content = fs::read_to_string(&output_path).unwrap();
		assert!(content.contains("@namespace1:"));
		assert!(content.contains("$var1 := %namespace2.var2"));
		assert!(content.contains("@namespace2:"));
		assert!(content.contains("$var2 := %namespace1.var1"));

		teardown(&output_path);
	}
}