#[cfg(test)]
mod runtime_tests {
    use vtc::parser::ast::{Accessor, Number, Reference, ReferenceType, Value};
    use vtc::parser::ast::Accessor::Index;
    use vtc::parser::ast::ReferenceType::Local;
    use vtc::runtime::runtime::{Runtime, RuntimeError};

    fn create_test_vtc() -> String {
        r#"
@Main:
    $target := "Local Target"
    $local_ref := %target
    $external_ref := &Other.target
    $nested_list := [1, [2, 3], 4]
    $chained_ref := %local_ref
    $multi_list := ["testing1", "testing2", "testing3"]
    $ref_with_accessor := %multi_list->(1)
    $ref_with_range := %multi_list->(0..2)

@Other:
    $target := "External Target"
"#
            .trim()
            .to_string()
    }

    #[test]
    fn test_load_vtc() {
        let mut runtime = Runtime::new();
        assert!(runtime.load_vtc(&create_test_vtc()).is_ok());
    }

    #[test]
    fn test_local_reference() {
        let mut runtime = Runtime::new();
        runtime.load_vtc(&create_test_vtc()).unwrap();

        let reference = Reference {
            ref_type: ReferenceType::Local,
            namespace: Some("Main".to_string()),
            variable: "local_ref".to_string(),
            accessors: vec![],
        };

        assert_eq!(
            runtime.get_value_with_ref(&reference).unwrap(),
            Value::String("Local Target".to_string())
        );
    }

    #[test]
    fn test_external_reference() {
        let mut runtime = Runtime::new();
        runtime.load_vtc(&create_test_vtc()).unwrap();

        let reference = Reference {
            ref_type: ReferenceType::Local,
            namespace: Some("Main".to_string()),
            variable: "external_ref".to_string(),
            accessors: vec![],
        };

        assert_eq!(
            runtime.get_value_with_ref(&reference).unwrap(),
            Value::String("External Target".to_string())
        );
    }

    #[test]
    fn test_nested_list_pointer() {
        let mut runtime = Runtime::new();
        runtime.load_vtc(&create_test_vtc()).unwrap();

        let reference = Reference {
            ref_type: Local,
            namespace: Some("Main".to_string()),
            variable: "nested_list".to_string(),
            accessors: vec![Index(1), Accessor::Index(0)],
        };

        let accessors = vec![Accessor::Index(1), Accessor::Index(0)];
        let test_value = runtime.get_value("Main", "nested_list", Local, accessors).unwrap();

        assert_eq!(
            test_value,
            Value::Number(Number::Integer(2))
        );
    }

    #[test]
    fn test_chained_references() {
        let mut runtime = Runtime::new();
        runtime.load_vtc(&create_test_vtc()).unwrap();

        let reference = Reference {
            ref_type: Local,
            namespace: Some("Main".to_string()),
            variable: "chained_ref".to_string(),
            accessors: vec![],
        };

        assert_eq!(
            runtime.get_value_with_ref(&reference).unwrap(),
            Value::String("Local Target".to_string())
        );
    }

    #[test]
    fn test_reference_with_accessor() {
        let mut runtime = Runtime::new();
        runtime.load_vtc(&create_test_vtc()).unwrap();

        // Test accessing a specific element (should return the element as is)
        let test_value_with_accessor = runtime.get_value("Main", "ref_with_accessor",
                                                         Local, vec![]).unwrap();
        assert_eq!(test_value_with_accessor, Value::String("testing2".to_string()));

        // Test accessing a range (should allow further accessors)
        let test_value_with_range = runtime.get_value("Main", "ref_with_range",
                                                      Local, vec![Index(1)]).unwrap();
        assert_eq!(test_value_with_range, Value::String("testing2".to_string()));

        // Test accessing the original list (should return the whole list)
        let original_list = runtime.get_value("Main", "multi_list",
                                              Local, vec![]).unwrap();
        assert_eq!(original_list, Value::List(vec![
            Value::String("testing1".to_string()),
            Value::String("testing2".to_string()),
            Value::String("testing3".to_string()),
        ]));

        // Test accessing an element of the original list
        let list_element = runtime.get_value("Main", "multi_list",
                                             Local, vec![Index(0)]).unwrap();
        assert_eq!(list_element, Value::String("testing1".to_string()));
    }

    #[test]
    fn test_invalid_vtc_syntax() {
        let invalid_vtc = r#"
@InvalidSyntax:
    $this_is_not_valid := [1, 2, 3
"#
            .trim()
            .to_string();

        let mut runtime = Runtime::new();
        let result = runtime.load_vtc(&invalid_vtc);
        assert!(result.is_err(), "Expected an error, but got: {:?}", result);
        match result {
            Err(RuntimeError::ParseError(_)) => (),
            _ => panic!("Expected ParseError, but got: {:?}", result),
        }
    }
}