#[cfg(test)]
mod tests {
	use vtc::parser::lexer::{Token, tokenize};

	#[test]
	fn test_lexer_boolean() {
		let test_input = r#"
		@testing_lexer:
			$var1 := True
			$var2 := False
            $var3 := [True, False]
        "#;

		let (_, tokens) = tokenize(test_input).unwrap();

		assert_eq!(
			tokens,
			vec![
				Token::Namespace(String::from("testing_lexer")),
				Token::Colon,
				Token::Variable(String::from("var1")),
				Token::Assign,
				Token::Boolean(true),

				Token::Variable(String::from("var2")),
				Token::Assign,
				Token::Boolean(false),

				Token::Variable(String::from("var3")),
				Token::Assign,
				Token::OpenBracket,
				Token::Boolean(true),
				Token::Comma,
				Token::Boolean(false),
				Token::CloseBracket
			]
		);
	}

	#[test]
	fn test_lexer_intrinsic() {
		let test_input = r#"
		@testing_lexer:
			$var1 := [add!!, 1, 2]
        "#;

		let (_, tokens) = tokenize(test_input).unwrap();

		assert_eq!(
			tokens,
			vec![
				Token::Namespace(String::from("testing_lexer")),
				Token::Colon,
				Token::Variable(String::from("var1")),
				Token::Assign,
				Token::OpenBracket,
				Token::Intrinsic(String::from("add")),
				Token::Comma,
				Token::Integer(1),
				Token::Comma,
				Token::Integer(2),
				Token::CloseBracket
			]
		);
	}
}