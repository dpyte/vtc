#[cfg(test)]
mod tests {
	use vtc::parser::lexer::{Token, tokenize, INLINE_CAPACITY};
	use std::sync::Arc;
	use smallvec::{smallvec, SmallVec};

	#[test]
	fn test_lexer_boolean() {
		let test_input = r#"
        @testing_lexer:
            $var1 := True
            $var2 := False
            $var3 := [True, False]
        "#;

		let (_, tokens) = tokenize(test_input).unwrap();

		let expected: SmallVec<[Token; INLINE_CAPACITY]> = smallvec![
            Token::Namespace(Arc::new(String::from("testing_lexer"))),
            Token::Colon,
            Token::Variable(Arc::new(String::from("var1"))),
            Token::Assign,
            Token::Boolean(true),

            Token::Variable(Arc::new(String::from("var2"))),
            Token::Assign,
            Token::Boolean(false),

            Token::Variable(Arc::new(String::from("var3"))),
            Token::Assign,
            Token::OpenBracket,
            Token::Boolean(true),
            Token::Comma,
            Token::Boolean(false),
            Token::CloseBracket
        ];

		assert_eq!(tokens, expected);
	}

	#[test]
	fn test_lexer_intrinsic() {
		let test_input = r#"
        @testing_lexer:
            $var1 := [add!!, 1, 2]
        "#;

		let (_, tokens) = tokenize(test_input).unwrap();

		let expected: SmallVec<[Token; INLINE_CAPACITY]> = smallvec![
            Token::Namespace(Arc::new(String::from("testing_lexer"))),
            Token::Colon,
            Token::Variable(Arc::new(String::from("var1"))),
            Token::Assign,
            Token::OpenBracket,
            Token::Intrinsic(Arc::new(String::from("add"))),
            Token::Comma,
            Token::Integer(1),
            Token::Comma,
            Token::Integer(2),
            Token::CloseBracket
        ];

		assert_eq!(tokens, expected);
	}
}