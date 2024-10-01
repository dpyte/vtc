extern crate core;

#[cfg(test)]
mod tests {
    use core::num::dec2flt::parse::parse_number;
    use vtc::parser::ast::{Accessor, Number, Reference, ReferenceType, Value};
    use vtc::parser::grammar::{parse};
    use vtc::parser::lexer::{tokenize, Token};

    #[test]
    fn test_parse_number() {
        let tokens = vec![Token::Integer(42)];
        assert_eq!(parse_number(&tokens), Ok((&[][..], Number::Integer(42))));

        let tokens = vec![Token::Float(3.14)];
        assert_eq!(parse_number(&tokens), Ok((&[][..], Number::Float(3.14))));

        let tokens = vec![Token::Binary(0b1010)];
        assert_eq!(parse_number(&tokens), Ok((&[][..], Number::Binary(10))));

        let tokens = vec![Token::Hexadecimal(0xFF)];
        assert_eq!(
            parse_number(&tokens),
            Ok((&[][..], Number::Hexadecimal(255)))
        );

        let tokens = vec![Token::String("not a number".to_string())];
        assert!(parse_number(&tokens).is_err());
    }

    #[test]
    fn test_parse_reference() {
        let tokens = vec![Token::Reference(
            "&namespace.variable->accessor[0..2]".to_string(),
        )];
        let expected = Reference {
            ref_type: ReferenceType::External,
            namespace: Some("namespace".to_string()),
            variable: "variable".to_string(),
            accessors: vec![Accessor::Key("accessor".to_string()), Accessor::Range(0, 2)],
        };
        assert_eq!(parse_reference(&tokens), Ok((&[][..], expected)));

        let tokens = vec![Token::Reference("%local_var".to_string())];
        let expected = Reference {
            ref_type: ReferenceType::Local,
            namespace: None,
            variable: "local_var".to_string(),
            accessors: vec![],
        };
        assert_eq!(parse_reference(&tokens), Ok((&[][..], expected)));

        let tokens = vec![Token::String("not a reference".to_string())];
        assert!(parse_reference(&tokens).is_err());
    }

    #[test]
    fn test_parse_list() {
        let tokens = vec![
            Token::OpenBracket,
            Token::Integer(1),
            Token::Comma,
            Token::Float(2.0),
            Token::Comma,
            Token::String("three".to_string()),
            Token::CloseBracket,
        ];
        let expected = vec![
            Value::Number(Number::Integer(1)),
            Value::Number(Number::Float(2.0)),
            Value::String("three".to_string()),
        ];
        assert_eq!(parse_list(&tokens), Ok((&[][..], expected)));

        let tokens = vec![Token::OpenBracket, Token::CloseBracket];
        assert_eq!(parse_list(&tokens), Ok((&[][..], vec![])));

        let tokens = vec![Token::OpenBracket, Token::Integer(1), Token::CloseBracket];
        assert_eq!(
            parse_list(&tokens),
            Ok((&[][..], vec![Value::Number(Number::Integer(1))]))
        );

        let tokens = vec![Token::OpenBracket, Token::Integer(1)];
        assert!(parse_list(&tokens).is_err());
    }

    #[test]
    fn test_parse_value() {
        let tokens = vec![Token::String("hello".to_string())];
        assert_eq!(
            parse_value(&tokens),
            Ok((&[][..], Value::String("hello".to_string())))
        );

        let tokens = vec![Token::Integer(42)];
        assert_eq!(
            parse_value(&tokens),
            Ok((&[][..], Value::Number(Number::Integer(42))))
        );

        let tokens = vec![Token::Boolean(true)];
        assert_eq!(parse_value(&tokens), Ok((&[][..], Value::Boolean(true))));

        let tokens = vec![Token::Nil];
        assert_eq!(parse_value(&tokens), Ok((&[][..], Value::Nil)));

        let tokens = vec![
            Token::OpenBracket,
            Token::Integer(1),
            Token::Comma,
            Token::Integer(2),
            Token::CloseBracket,
        ];
        assert_eq!(
            parse_value(&tokens),
            Ok((
                &[][..],
                Value::List(vec![
                    Value::Number(Number::Integer(1)),
                    Value::Number(Number::Integer(2)),
                ])
            ))
        );

        let tokens = vec![Token::Reference("%local_var".to_string())];
        let expected = Value::Reference(Reference {
            ref_type: ReferenceType::Local,
            namespace: None,
            variable: "local_var".to_string(),
            accessors: vec![],
        });
        assert_eq!(parse_value(&tokens), Ok((&[][..], expected)));
    }

    #[test]
    fn test_parse_vtc_config() {
        let input = r#"
@Engine_Import:
    $device_type := ["DEVICE_NAME_GOES_HERE", "EXAMPLE"]
    $enable_features := [
        "native",
        "wireless",
        "remote_control",
        %device_type->(1)
    ]

@test_balanced:
    $pass := ["None"]
    $erro := []
    $check := [
        %pass->(0),
        "Test"
    ]
    $fail := [
        %check->[Test],
        %check->(1)
    ]

@test_reference:
    $var_1 := [
        "Hello",
        "World"
    ]
    $var_2 := %var_1->(0)
    $var_4 := &test_reference.var_1->(0)
    $var_6 := Nil
    $var_7 := "Hello, World"
    $var_8 := %var_1->(1)
"#;

        match tokenize(input) {
            Ok((remaining, tokens)) => {
                println!("Tokens:");
                for (index, token) in tokens.iter().enumerate() {
                    println!("{}: {:?}", index, token);
                }
                println!("Remaining input: {:?}", remaining);
            }
            Err(e) => println!("Error: {:?}", e),
        }

        let (remaining_tokens, tokens) = tokenize(input).unwrap();
        let vtc_file = match parse(&tokens) {
            Ok(vtc_file) => {
                println!("Successfully parsed VTC file: {:?}", vtc_file);
                vtc_file
            }
            Err(e) => {
                println!("Error parsing VTC file: {:?}", e);
                return;
            }
        };

        // Assert that all tokens were consumed
        assert!(remaining_tokens.is_empty(), "Not all tokens were consumed");

        // assert_eq!(vtc_file.namespaces.len(), 3);

        let engine_import = &vtc_file.1.namespaces[0]; //  namespaces[0];
        assert_eq!(engine_import.name, "Engine_Import");
        assert_eq!(engine_import.variables.len(), 2);

        let device_type = &engine_import.variables[0];
        assert_eq!(device_type.name, "device_type");
        assert!(matches!(device_type.value, Value::List(_)));

        let enable_features = &engine_import.variables[1];
        assert_eq!(enable_features.name, "enable_features");
        assert!(matches!(enable_features.value, Value::List(_)));

        // Check the test_balanced namespace
        let test_balanced = &vtc_file.1.namespaces[1];
        assert_eq!(test_balanced.name, "test_balanced");
        assert_eq!(test_balanced.variables.len(), 4);

        // Check the test_reference namespace
        let test_reference = &vtc_file.1.namespaces[2];
        assert_eq!(test_reference.name, "test_reference");
        assert_eq!(test_reference.variables.len(), 6);

        // Let's check a specific variable in more detail
        let var_4 = &test_reference.variables[3];
        assert_eq!(var_4.name, "var_6");
        assert!(matches!(var_4.value, Value::Nil));
        if let Value::Reference(ref r) = var_4.value {
            assert_eq!(r.ref_type, ReferenceType::External);
            assert_eq!(r.namespace, Some("test_reference".to_string()));
            assert_eq!(r.variable, "var_1");
            assert_eq!(r.accessors, vec![Accessor::Index(0)]);
        }
    }

    #[test]
    fn test_parse_range_operator() {
        let input = r#"
        @test_sample:
            $value_1 := ["hello", "world", "\0"]
            $value_2 := [True, False, %test_sample.value_1->(0..2), %value_2->(0..2)]
        "#;
        match tokenize(input) {
            Ok((remaining, tokens)) => {
                println!("Tokens:");
                for (index, token) in tokens.iter().enumerate() {
                    println!("{}: {:?}", index, token);
                }
                println!("Remaining input: {:?}", remaining);
            }
            Err(e) => println!("Error: {:?}", e),
        }

        let (remaining_tokens, tokens) = tokenize(input).unwrap();
        let vtc_file = match parse(&tokens) {
            Ok(vtc_file) => {
                println!("Successfully parsed VTC file: {:?}", vtc_file);
                vtc_file
            }
            Err(e) => {
                println!("Error parsing VTC file: {:?}", e);
                return;
            }
        };
    }
}
