use std::sync::Arc;

use smallvec::SmallVec;

use crate::parser::lexer::Token;
use crate::value::{Accessor, Namespace, Number, Reference, ReferenceType, Value, Variable, VtcFile};

const SMALL_VEC_SIZE: usize = 4;

pub struct Parser<'a> {
	tokens: &'a [Token],
	position: usize,
}

impl<'a> Parser<'a> {
	pub fn new(tokens: &'a [Token]) -> Self {
		Parser {
			tokens,
			position: 0,
		}
	}

	pub fn parse(&mut self) -> Result<VtcFile, String> {
		let mut namespaces = Vec::new();
		while self.position < self.tokens.len() {
			namespaces.push(self.parse_namespace()?);
		}
		Ok(VtcFile { namespaces })
	}

	fn parse_namespace(&mut self) -> Result<Namespace, String> {
		self.expect_token(|t| matches!(t, Token::Namespace(_)))?;
		let name = match &self.tokens[self.position - 1] {
			Token::Namespace(n) => n.clone(),
			_ => unreachable!(),
		};
		self.expect_token(|t| *t == Token::Colon)?;

		let mut variables = Vec::new();
		while self.peek_token()
			.map_or(false, |t| matches!(t, Token::Variable(_)))
		{
			variables.push(self.parse_variable()?);
		}

		Ok(Namespace { name: name.to_string(), variables })
	}

	fn parse_variable(&mut self) -> Result<Variable, String> {
		self.expect_token(|t| matches!(t, Token::Variable(_)))?;
		let name = match &self.tokens[self.position - 1] {
			Token::Variable(n) => n.clone(),
			_ => unreachable!(),
		};
		self.expect_token(|t| *t == Token::Assign)?;
		let value = self.parse_value()?;
		Ok(Variable { name: name.to_string(), value })
	}

	fn parse_value(&mut self) -> Result<Value, String> {
		match self.next_token() {
			Some(token) => match token {
				Token::OpenBracket => self.parse_list(),
				Token::Intrinsic(i) => Ok(Value::Intrinsic(i.to_string())),
				Token::String(s) => Ok(Value::String(s.to_string())),
				Token::Integer(i) => Ok(Value::Number(Number::Integer(*i))),
				Token::Float(f) => Ok(Value::Number(Number::Float(*f))),
				Token::Binary(b) => Ok(Value::Number(Number::Binary(*b))),
				Token::Hexadecimal(h) => Ok(Value::Number(Number::Hexadecimal(*h))),
				Token::Boolean(b) => Ok(Value::Boolean(*b)),
				Token::Nil => Ok(Value::Nil),
				Token::Reference(_) => self.parse_reference(),
				_ => Err(format!("Unexpected token when parsing value: {:?}", token)),
			},
			None => Err("Unexpected end of input when parsing value".to_string()),
		}
	}

	fn parse_list(&mut self) -> Result<Value, String> {
		let mut values = Vec::new();
		loop {
			if self.peek_token() == Some(&Token::CloseBracket) {
				self.next_token();
				break;
			}

			values.push(self.parse_value()?);
			match self.peek_token() {
				Some(&Token::Comma) => {
					self.next_token();
				}
				Some(&Token::CloseBracket) => {}
				_ => return Err("Expected ',' or ']'".to_string()),
			}
		}
		Ok(Value::List(Arc::new(values)))  // Only Arc the Vec itself
	}

	fn parse_reference(&mut self) -> Result<Value, String> {
		let reference_token = match &self.tokens[self.position - 1] {
			Token::Reference(r) => r,
			_ => return Err("Expected reference".to_string()),
		};

		let (ref_type, rest) = if reference_token.starts_with('&') {
			(ReferenceType::External, &reference_token[1..])
		} else if reference_token.starts_with('%') {
			(ReferenceType::Local, &reference_token[1..])
		} else {
			return Err("Invalid reference type".to_string());
		};

		let parts: Vec<&str> = rest.split('.').collect();
		let (namespace, variable) = match parts.len() {
			1 => (None, Arc::new(parts[0].to_string())),
			2 => (Some(Arc::new(parts[0].to_string())), Arc::new(parts[1].to_string())),
			_ => return Err("Invalid reference format".to_string()),
		};

		let mut accessors = SmallVec::new();
		while self.peek_token() == Some(&Token::Pointer) {
			self.next_token();
			accessors.push(self.parse_accessor()?);
		}

		Ok(Value::Reference(Reference {
			ref_type,
			namespace,
			variable,
			accessors,
		}))
	}

	fn parse_accessor(&mut self) -> Result<Accessor, String> {
		match self.next_token() {
			Some(token) => match token {
				Token::OpenParen => self.parse_index_or_range(),
				Token::OpenBracket => self.parse_key(),
				_ => Err(format!("Expected accessor, found {:?}", token)),
			},
			None => Err("Unexpected end of input when parsing accessor".to_string()),
		}
	}

	fn parse_index_or_range(&mut self) -> Result<Accessor, String> {
		let start = self.expect_token(|t| matches!(t, Token::Integer(_)))?;
		if self.peek_token() == Some(&Token::Range) {
			self.next_token();
			let end = self.expect_token(|t| matches!(t, Token::Integer(_)))?;
			self.expect_token(|t| *t == Token::CloseParen)?;
			Ok(Accessor::Range(
				start.parse::<usize>().unwrap(),
				end.parse::<usize>().unwrap(),
			))
		} else {
			self.expect_token(|t| *t == Token::CloseParen)?;
			Ok(Accessor::Index(start.parse::<usize>().unwrap()))
		}
	}

	fn parse_key(&mut self) -> Result<Accessor, String> {
		let key = self.expect_token(|t| matches!(t, Token::String(_) | Token::Identifier(_)))?;
		self.expect_token(|t| *t == Token::CloseBracket)?;
		Ok(Accessor::Key(key))
	}

	fn next_token(&mut self) -> Option<&Token> {
		while self.position < self.tokens.len() {
			let token = &self.tokens[self.position];
			self.position += 1;
			if !matches!(token, Token::Comment(_)) {
				return Some(token);
			}
		}
		None
	}

	fn peek_token(&self) -> Option<&Token> {
		let mut pos = self.position;
		while pos < self.tokens.len() {
			let token = &self.tokens[pos];
			if !matches!(token, Token::Comment(_)) {
				return Some(token);
			}
			pos += 1;
		}
		None
	}

	fn expect_token<F>(&mut self, predicate: F) -> Result<String, String>
	where
		F: Fn(&Token) -> bool,
	{
		match self.next_token() {
			Some(token) if predicate(token) => match token {
				Token::Namespace(s)
				| Token::Variable(s)
				| Token::String(s)
				| Token::Identifier(s) => Ok(s.to_string()),
				Token::Integer(i) => Ok(i.to_string()),
				Token::Float(f) => Ok(f.to_string()),
				Token::Binary(b) => Ok(format!("0b{:b}", b)),
				Token::Hexadecimal(h) => Ok(format!("0x{:X}", h)),
				Token::Boolean(b) => Ok(b.to_string()),
				_ => Ok(format!("{:?}", token)),
			},
			_ => Err("Unexpected token".to_string()),
		}
	}
}

pub fn parse(tokens: &[Token]) -> Result<VtcFile, String> {
	let mut parser = Parser::new(tokens);
	parser.parse()
}
