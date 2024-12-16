use std::str::Chars;
use std::iter::Peekable;
use smallvec::SmallVec;
use fnv::FnvHashMap;

pub const SMALL_VEC_SIZE: usize = 128;

#[derive(Debug, PartialEq, Clone)]
pub enum Token {
	Namespace(String),
	Variable(String),
	Assign,
	OpenBracket,
	CloseBracket,
	OpenParen,
	CloseParen,
	Comma,
	String(String),
	Integer(i64),
	Float(f64),
	Binary(i64),
	Hexadecimal(i64),
	Boolean(bool),
	Nil,
	Reference(String),
	Pointer,
	Dot,
	Range,
	Identifier(String),
	Colon,
	Intrinsic(String),
	Comment(String),
}

#[derive(Debug)]
pub struct Lexer<'a> {
	input: Peekable<Chars<'a>>,
	position: usize,
	keywords: FnvHashMap<&'static str, Token>,
}

/// Tokenizes the input string and returns remaining input and tokens
pub fn tokenize(input: &str) -> Result<(&str, SmallVec<[Token; SMALL_VEC_SIZE]>), LexerError> {
	let mut lexer = Lexer::new(input);
	let tokens = lexer.tokenize()?;
	let remaining = &input[lexer.position..];
	Ok((remaining, tokens))
}

#[derive(Debug, thiserror::Error)]
pub enum LexerError {
	#[error("Unexpected character: {0}")]
	UnexpectedChar(char),
	#[error("Invalid number format: {0}")]
	InvalidNumber(String),
	#[error("Unterminated string")]
	UnterminatedString,
	#[error("Invalid identifier: {0}")]
	InvalidIdentifier(String),
}

impl<'a> Lexer<'a> {
	pub fn new(input: &'a str) -> Self {
		let mut keywords = FnvHashMap::default();
		keywords.insert("True", Token::Boolean(true));
		keywords.insert("False", Token::Boolean(false));
		keywords.insert("Nil", Token::Nil);

		Self {
			input: input.chars().peekable(),
			position: 0,
			keywords,
		}
	}

	pub fn tokenize(&mut self) -> Result<SmallVec<[Token; SMALL_VEC_SIZE]>, LexerError> {
		let mut tokens = SmallVec::new();

		while let Some(&c) = self.input.peek() {
			let token = match c {
				c if c.is_whitespace() => {
					self.consume_char();
					continue;
				}
				'@' => {
					self.consume_char();
					Token::Namespace(self.read_identifier()?)
				}
				'$' => {
					self.consume_char();
					Token::Variable(self.read_identifier()?)
				}
				'[' => {
					self.consume_char();
					Token::OpenBracket
				}
				']' => {
					self.consume_char();
					Token::CloseBracket
				}
				'(' => {
					self.consume_char();
					Token::OpenParen
				}
				')' => {
					self.consume_char();
					Token::CloseParen
				}
				',' => {
					self.consume_char();
					Token::Comma
				}
				':' => {
					self.consume_char();
					if self.peek_char_eq('=') {
						self.consume_char();
						Token::Assign
					} else {
						Token::Colon
					}
				}
				'#' => {
					self.consume_char();
					Token::Comment(self.read_until('\n'))
				}
				'-' => {
					self.consume_char();
					if self.peek_char_eq('>') {
						self.consume_char();
						Token::Pointer
					} else if self.peek_is_digit() {
						let num = self.read_number('-')?;
						num
					} else {
						return Err(LexerError::UnexpectedChar('-'));
					}
				}
				'.' => {
					self.consume_char();
					if self.peek_char_eq('.') {
						self.consume_char();
						Token::Range
					} else {
						Token::Dot
					}
				}
				'&' | '%' => {
					let ref_type = self.consume_char();
					Token::Reference(format!("{}{}", ref_type, self.read_reference()?))
				}
				'"' | '\'' => self.read_string()?,
				'0' => {
					self.consume_char();
					match self.peek() {
						Some('b') => self.read_binary()?,
						Some('x') => self.read_hexadecimal()?,
						Some(c) if c.is_digit(10) => self.read_number('0')?,
						_ => Token::Integer(0),
					}
				}
				c if c.is_digit(10) => self.read_number(c)?,
				c if c.is_alphabetic() || c == '_' => {
					let ident = self.read_identifier()?;
					if let Some(token) = self.keywords.get(ident.as_str()).cloned() {
						token
					} else if self.peek_chars_eq("!!") {
						self.consume_chars(2);
						Token::Intrinsic(ident)
					} else {
						Token::Identifier(ident)
					}
				}
				_ => return Err(LexerError::UnexpectedChar(c)),
			};

			tokens.push(token);
		}

		Ok(tokens)
	}

	fn consume_char(&mut self) -> char {
		self.position += 1;
		self.input.next().unwrap()
	}

	fn peek(&mut self) -> Option<char> {
		self.input.peek().copied()
	}

	fn peek_char_eq(&mut self, c: char) -> bool {
		self.peek().map_or(false, |p| p == c)
	}

	fn peek_chars_eq(&mut self, s: &str) -> bool {
		let mut chars = self.input.clone();
		s.chars().all(|c| chars.next() == Some(c))
	}

	fn peek_is_digit(&mut self) -> bool {
		self.peek().map_or(false, |c| c.is_digit(10))
	}

	fn consume_chars(&mut self, n: usize) {
		for _ in 0..n {
			self.consume_char();
		}
	}

	fn read_until(&mut self, delimiter: char) -> String {
		let mut result = String::new();
		while let Some(&c) = self.input.peek() {
			if c == delimiter {
				break;
			}
			result.push(self.consume_char());
		}
		result
	}

	fn read_identifier(&mut self) -> Result<String, LexerError> {
		let mut identifier = String::new();

		if let Some(&c) = self.input.peek() {
			if !c.is_alphabetic() && c != '_' {
				return Err(LexerError::InvalidIdentifier(c.to_string()));
			}
		}

		while let Some(&c) = self.input.peek() {
			if !c.is_alphanumeric() && c != '_' {
				break;
			}
			identifier.push(self.consume_char());
		}

		if identifier.is_empty() {
			Err(LexerError::InvalidIdentifier("".to_string()))
		} else {
			Ok(identifier)
		}
	}

	fn read_string(&mut self) -> Result<Token, LexerError> {
		let quote = self.consume_char();
		let mut string = String::new();

		while let Some(&c) = self.input.peek() {
			if c == quote {
				self.consume_char();
				return Ok(Token::String(string));
			}
			string.push(self.consume_char());
		}

		Err(LexerError::UnterminatedString)
	}

	fn read_number(&mut self, first: char) -> Result<Token, LexerError> {
		let mut number = String::new();
		if first == '-' || first.is_digit(10) {
			number.push(first);
		}

		let mut has_next_digit = false;
		let mut is_float = false;

		while let Some(&c) = self.input.peek() {
			match c {
				'0'..='9' => {
					has_next_digit = true;
					number.push(self.consume_char());
				}
				'.' if !is_float => {
					is_float = true;
					number.push(self.consume_char());
					has_next_digit = false;
				}
				// Break on any non-numeric character
				_ => break,
			}
		}

		// Validate float format
		if is_float && !has_next_digit {
			return Err(LexerError::InvalidNumber(number));
		}

		if is_float {
			number.parse::<f64>()
				.map(Token::Float)
				.map_err(|_| LexerError::InvalidNumber(number))
		} else {
			number.parse::<i64>()
				.map(Token::Integer)
				.map_err(|_| LexerError::InvalidNumber(number))
		}
	}

	fn read_binary(&mut self) -> Result<Token, LexerError> {
		self.consume_char(); // consume 'b'
		let mut number = String::new();

		while let Some(&c) = self.input.peek() {
			if c != '0' && c != '1' {
				break;
			}
			number.push(self.consume_char());
		}

		i64::from_str_radix(&number, 2)
			.map(Token::Binary)
			.map_err(|_| LexerError::InvalidNumber(format!("0b{}", number)))
	}

	fn read_hexadecimal(&mut self) -> Result<Token, LexerError> {
		self.consume_char(); // consume 'x'
		let mut number = String::new();

		while let Some(&c) = self.input.peek() {
			if !c.is_digit(16) {
				break;
			}
			number.push(self.consume_char());
		}

		i64::from_str_radix(&number, 16)
			.map(Token::Hexadecimal)
			.map_err(|_| LexerError::InvalidNumber(format!("0x{}", number)))
	}

	fn read_reference(&mut self) -> Result<String, LexerError> {
		let mut reference = String::new();
		let mut has_pointer = false;
		let mut has_accessor = false;

		while let Some(&c) = self.input.peek() {
			match c {
				c if c.is_alphanumeric() || c == '_' || c == '.' => {
					reference.push(self.consume_char());
				}
				'-' if self.peek_chars_eq("->") => {
					has_pointer = true;
					reference.push_str("->");
					self.consume_chars(2);
				}
				'[' if has_pointer && !has_accessor => {
					has_accessor = true;
					reference.push(self.consume_char());
					reference.push_str(&self.read_until(']'));
					reference.push(self.consume_char());
				}
				'(' if has_pointer && !has_accessor => {
					has_accessor = true;
					reference.push(self.consume_char());
					reference.push_str(&self.read_until(')'));
					reference.push(self.consume_char());
				}
				_ => break,
			}
		}

		if reference.is_empty() {
			Err(LexerError::InvalidIdentifier("".to_string()))
		} else {
			Ok(reference)
		}
	}
}