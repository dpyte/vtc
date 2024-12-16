#[cfg(test)]
mod tests {
	use smallvec::SmallVec;
	use vtc::parser::lexer::{Token, tokenize, SMALL_VEC_SIZE, Lexer};

	#[test]
	fn test_basic_tokens() {
		let input = "@namespace $variable := [1, 2.0, True, False, Nil]";
		let mut lexer = Lexer::new(input);
		let tokens = lexer.tokenize().unwrap();

		assert_eq!(tokens[0], Token::Namespace("namespace".to_string()));
		assert_eq!(tokens[1], Token::Variable("variable".to_string()));
		assert_eq!(tokens[2], Token::Assign);
		assert_eq!(tokens[3], Token::OpenBracket);
		assert_eq!(tokens[4], Token::Integer(1));
		assert_eq!(tokens[5], Token::Comma);
		assert_eq!(tokens[6], Token::Float(2.0));
		assert_eq!(tokens[7], Token::Comma);
		assert_eq!(tokens[8], Token::Boolean(true));
		assert_eq!(tokens[9], Token::Comma);
		assert_eq!(tokens[10], Token::Boolean(false));
		assert_eq!(tokens[11], Token::Comma);
		assert_eq!(tokens[12], Token::Nil);
		assert_eq!(tokens[13], Token::CloseBracket);
	}

	#[test]
	fn test_references() {
		let input = "%local.var->(0) &external.var->[key]";
		let mut lexer = Lexer::new(input);
		let tokens = lexer.tokenize().unwrap();

		assert_eq!(tokens[0], Token::Reference("%local.var->(0)".to_string()));
		assert_eq!(tokens[1], Token::Reference("&external.var->[key]".to_string()));
	}

	#[test]
	fn test_numbers() {
		let input = "42 -1 3.14 0b1010 0xFF";
		let mut lexer = Lexer::new(input);
		let tokens = lexer.tokenize().unwrap();

		assert_eq!(tokens[0], Token::Integer(42));
		assert_eq!(tokens[1], Token::Integer(-1));
		assert_eq!(tokens[2], Token::Float(3.14));
		assert_eq!(tokens[3], Token::Binary(10));
		assert_eq!(tokens[4], Token::Hexadecimal(255));
	}

	#[test]
	fn test_lexer_boolean() {
		let test_input = r#"
        @testing_lexer:
            $var1 := True
            $var2 := False
            $var3 := [True, False]
        "#;

		let (_, tokens) = tokenize(test_input).unwrap();

		let expected: SmallVec<[Token; SMALL_VEC_SIZE]> = SmallVec::from_vec(vec![
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
		]);

		assert_eq!(tokens, expected);
	}

	#[test]
	fn test_lexer_intrinsic() {
		let test_input = r#"
        @testing_lexer:
            $var1 := [add!!, 1, 2]
        "#;

		let (_, tokens) = tokenize(test_input).unwrap();

		let expected: SmallVec<[Token; SMALL_VEC_SIZE]> = SmallVec::from_vec(vec![
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
		]);

		assert_eq!(tokens, expected);
	}
}