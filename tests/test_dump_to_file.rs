use std::fs;
use std::path::PathBuf;
use std::rc::Rc;


#[cfg(test)]
mod tests {
	use vtc::runtime::Runtime;
	use vtc::value::{Accessor, Number, Reference, ReferenceType, Value};
	use super::*;

	fn setup() -> (Runtime, PathBuf) {
		let runtime = Runtime::new();
		let output_path = PathBuf::from("test_output.vtc");
		(runtime, output_path)
	}

	fn teardown(path: &PathBuf) {
		fs::remove_file(path).unwrap_or_default();
	}

	#[test]
	fn test_empty_runtime() {
		let (runtime, output_path) = setup();

		runtime.dump_to_file(&output_path).unwrap();

		let content = fs::read_to_string(&output_path).unwrap();
		assert!(content.is_empty(), "File should be empty for an empty runtime");

		teardown(&output_path);
	}

	#[test]
	fn test_single_namespace_single_variable() {
		let (mut runtime, output_path) = setup();

		runtime.namespaces.insert(
			Rc::new("test_namespace".to_string()),
			{
				let mut map = std::collections::HashMap::new();
				map.insert(
					Rc::new("test_var".to_string()),
					Rc::new(Value::String(Rc::new("Hello, World!".to_string())))
				);
				map
			}
		);

		runtime.dump_to_file(&output_path).unwrap();

		let content = fs::read_to_string(&output_path).unwrap();
		assert_eq!(content, "@test_namespace:\n    $test_var := \"Hello, World!\"\n\n");

		teardown(&output_path);
	}

	#[test]
	fn test_multiple_namespaces_multiple_variables() {
		let (mut runtime, output_path) = setup();

		runtime.namespaces.insert(
			Rc::new("namespace1".to_string()),
			{
				let mut map = std::collections::HashMap::new();
				map.insert(Rc::new("var1".to_string()), Rc::new(Value::Number(Number::Integer(42))));
				map.insert(Rc::new("var2".to_string()), Rc::new(Value::Boolean(true)));
				map
			}
		);

		runtime.namespaces.insert(
			Rc::new("namespace2".to_string()),
			{
				let mut map = std::collections::HashMap::new();
				map.insert(Rc::new("var3".to_string()), Rc::new(Value::Number(Number::Float(3.14))));
				map
			}
		);

		runtime.dump_to_file(&output_path).unwrap();

		let content = fs::read_to_string(&output_path).unwrap();
		assert!(content.contains("@namespace1:"));
		assert!(content.contains("$var1 := 42"));
		assert!(content.contains("$var2 := true"));
		assert!(content.contains("@namespace2:"));
		assert!(content.contains("$var3 := 3.14"));

		teardown(&output_path);
	}

	#[test]
	fn test_list_values() {
		let (mut runtime, output_path) = setup();

		runtime.namespaces.insert(
			Rc::new("list_namespace".to_string()),
			{
				let mut map = std::collections::HashMap::new();
				map.insert(
					Rc::new("list_var".to_string()),
					Rc::new(Value::List(Rc::new(vec![
						Rc::new(Value::Number(Number::Integer(1))),
						Rc::new(Value::String(Rc::new("two".to_string()))),
						Rc::new(Value::Boolean(true))
					])))
				);
				map
			}
		);

		runtime.dump_to_file(&output_path).unwrap();

		let content = fs::read_to_string(&output_path).unwrap();
		assert!(content.contains("$list_var := [1, \"two\", true]"));

		teardown(&output_path);
	}

	#[test]
	fn test_reference_values() {
		let (mut runtime, output_path) = setup();

		runtime.namespaces.insert(
			Rc::new("ref_namespace".to_string()),
			{
				let mut map = std::collections::HashMap::new();
				map.insert(
					Rc::new("ref_var".to_string()),
					Rc::new(Value::Reference(Rc::new(Reference {
						ref_type: ReferenceType::Local,
						namespace: Some(Rc::new("other_namespace".to_string())),
						variable: Rc::new("other_var".to_string()),
						accessors: smallvec::smallvec![
                            Accessor::Index(0),
                            Accessor::Range(1, 3),
                            Accessor::Key(Rc::from("key".to_string()))
                        ]
					})))
				);
				map
			}
		);

		runtime.dump_to_file(&output_path).unwrap();

		let content = fs::read_to_string(&output_path).unwrap();
		assert!(content.contains("$ref_var := %other_namespace.other_var->(0)->(1, 3)->[key]"));

		teardown(&output_path);
	}

	#[test]
	fn test_intrinsic_values() {
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

		runtime.dump_to_file(&output_path).unwrap();

		let content = fs::read_to_string(&output_path).unwrap();
		assert!(content.contains("$intrinsic_var := std_add_int!!"));

		teardown(&output_path);
	}

	#[test]
	fn test_overwrite_existing_file() {
		let (mut runtime, output_path) = setup();

		// Write some initial content
		runtime.namespaces.insert(
			Rc::new("initial".to_string()),
			{
				let mut map = std::collections::HashMap::new();
				map.insert(Rc::new("var".to_string()), Rc::new(Value::String(Rc::new("initial".to_string()))));
				map
			}
		);
		runtime.dump_to_file(&output_path).unwrap();

		// Overwrite with new content
		runtime.namespaces.clear();
		runtime.namespaces.insert(
			Rc::new("overwritten".to_string()),
			{
				let mut map = std::collections::HashMap::new();
				map.insert(Rc::new("var".to_string()), Rc::new(Value::String(Rc::new("overwritten".to_string()))));
				map
			}
		);
		runtime.dump_to_file(&output_path).unwrap();

		let content = fs::read_to_string(&output_path).unwrap();
		assert!(content.contains("@overwritten:"));
		assert!(content.contains("$var := \"overwritten\""));
		assert!(!content.contains("@initial:"));
		assert!(!content.contains("$var := \"initial\""));

		teardown(&output_path);
	}

	#[test]
	fn test_large_runtime() {
		let (mut runtime, output_path) = setup();

		for i in 0..1000 {
			let namespace = format!("namespace_{}", i);
			runtime.namespaces.insert(
				Rc::new(namespace),
				{
					let mut map = std::collections::HashMap::new();
					for j in 0..10 {
						let var_name = format!("var_{}", j);
						let value = Value::Number(Number::Integer((i * 10 + j) as i64));
						map.insert(Rc::new(var_name), Rc::new(value));
					}
					map
				}
			);
		}

		runtime.dump_to_file(&output_path).unwrap();

		let content = fs::read_to_string(&output_path).unwrap();
		assert!(content.contains("@namespace_0:"));
		assert!(content.contains("@namespace_999:"));
		assert!(content.contains("$var_0 := 0"));
		assert!(content.contains("$var_9 := 9999"));

		teardown(&output_path);
	}
}