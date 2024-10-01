#[cfg(test)]
mod runtime_tests {
	use std::path::PathBuf;
	use vtc::parser::ast::ReferenceType::{External, Local};
	use vtc::parser::ast::Value::{List, String};
	use vtc::parser::ast::{
		Accessor, Namespace, Number, Reference, ReferenceType, Value, Variable, VtcFile,
	};
	use vtc::runtime::runtime::{Runtime, RuntimeError};

	fn create_test_vtc_file() -> VtcFile {
        VtcFile {
	        namespaces: vec![Namespace {
		        name: "Test".to_string(),
		        variables: vec![
			        Variable {
				        name: "string_var".to_string(),
				        value: Value::String("Hello, World!".to_string()),
			        },
			        Variable {
				        name: "int_var".to_string(),
				        value: Value::Number(Number::Integer(42)),
			        },
			        Variable {
				        name: "float_var".to_string(),
				        value: Value::Number(Number::Float(3.14)),
			        },
			        Variable {
				        name: "bool_var".to_string(),
				        value: Value::Boolean(true),
			        },
			        Variable {
				        name: "list_var".to_string(),
				        value: Value::List(vec![
					        Value::Number(Number::Integer(1)),
					        Value::Number(Number::Integer(2)),
					        Value::Number(Number::Integer(3)),
				        ]),
			        },
		        ],
	        }],
        }
    }

    #[test]
    fn test_load_vtc_file() {
        let mut runtime = Runtime::new();
        let vtc_file = create_test_vtc_file();
        assert!(runtime.load_vtc_file(vtc_file).is_ok());
    }

    #[test]
    fn test_get_string_value() {
        let mut runtime = Runtime::new();
        runtime.load_vtc_file(create_test_vtc_file()).unwrap();

        let reference = Reference {
            ref_type: ReferenceType::Local,
            namespace: Some("Test".to_string()),
            variable: "string_var".to_string(),
            accessors: vec![],
        };

        assert_eq!(
            runtime.get_value_with_ref(&reference).unwrap(),
            Value::String("Hello, World!".to_string())
        );
    }

    #[test]
    fn test_get_integer_value() {
        let mut runtime = Runtime::new();
        runtime.load_vtc_file(create_test_vtc_file()).unwrap();

        let reference = Reference {
            ref_type: ReferenceType::Local,
            namespace: Some("Test".to_string()),
            variable: "int_var".to_string(),
            accessors: vec![],
        };

        assert_eq!(
            runtime.get_value_with_ref(&reference).unwrap(),
            Value::Number(Number::Integer(42))
        );
    }

    #[test]
    fn test_get_list_value_with_index() {
        let mut runtime = Runtime::new();
        runtime.load_vtc_file(create_test_vtc_file()).unwrap();

        let reference = Reference {
            ref_type: ReferenceType::Local,
            namespace: Some("Test".to_string()),
            variable: "list_var".to_string(),
            accessors: vec![Accessor::Index(1)],
        };

        let test_value = runtime.get_value_with_ref(&reference).unwrap();

	    assert_eq!(test_value, Value::Number(Number::Integer(2)));
    }

    #[test]
    fn test_get_list_value_with_range() {
        let mut runtime = Runtime::new();
        runtime.load_vtc_file(create_test_vtc_file()).unwrap();

        let reference = Reference {
            ref_type: ReferenceType::Local,
            namespace: Some("Test".to_string()),
            variable: "list_var".to_string(),
            accessors: vec![Accessor::Range(0, 2)],
        };

        assert_eq!(
            runtime.get_value_with_ref(&reference).unwrap(),
            Value::List(vec![
                Value::Number(Number::Integer(1)),
                Value::Number(Number::Integer(2)),
            ])
        );
    }

    #[test]
    fn test_variable_not_found() {
        let mut runtime = Runtime::new();
        runtime.load_vtc_file(create_test_vtc_file()).unwrap();

        let reference = Reference {
            ref_type: ReferenceType::Local,
            namespace: Some("Test".to_string()),
            variable: "non_existent_var".to_string(),
            accessors: vec![],
        };

        assert!(matches!(
            runtime.get_value_with_ref(&reference),
            Err(RuntimeError::VariableNotFound(_))
        ));
    }

    #[test]
    fn test_namespace_not_found() {
        let mut runtime = Runtime::new();
        runtime.load_vtc_file(create_test_vtc_file()).unwrap();

        let reference = Reference {
            ref_type: ReferenceType::External,
            namespace: Some("NonExistentNamespace".to_string()),
            variable: "some_var".to_string(),
            accessors: vec![],
        };

        assert!(matches!(
            runtime.get_value_with_ref(&reference),
            Err(RuntimeError::NamespaceNotFound(_))
        ));
    }

    #[test]
    fn test_invalid_accessor() {
        let mut runtime = Runtime::new();
        runtime.load_vtc_file(create_test_vtc_file()).unwrap();

        let reference = Reference {
            ref_type: ReferenceType::Local,
            namespace: Some("Test".to_string()),
            variable: "string_var".to_string(),
            accessors: vec![Accessor::Index(0)],
        };

        assert!(matches!(
            runtime.get_value_with_ref(&reference),
            Err(RuntimeError::InvalidAccessor(_))
        ));
    }

    #[test]
    fn test_read_file() {
        let mut runtime = Runtime::new();
	    runtime
		    .read_file(PathBuf::from("./tests/inherit_directive.vtc".to_string()))
		    .unwrap();

	    let test_sample_value_1 = runtime
		    .get_value("test_sample", "value_1", Local, vec![])
		    .unwrap();
	    assert_eq!(
		    test_sample_value_1,
		    List(vec![
			    String("hello".to_string()),
			    String("world".to_string()),
			    String("\\0".to_string()),
		    ])
	    );

	    let inherit_value_1 = runtime
		    .get_value("test_inherit", "inherit_1", External, vec![])
		    .unwrap();
	    assert_eq!(
		    inherit_value_1,
		    List(vec![
			    String("world".to_string()),
			    List(vec![
				    String("hello".to_string()),
				    String("world".to_string())
			    ]),
			    String("Testing limit in directive test_inherit.".to_string()),
		    ])
	    );
    }
}
