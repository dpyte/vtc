use nom::{
    branch::alt,
    bytes::complete::{tag, take_until, take_while1},
    character::complete::{alpha1, alphanumeric1, char, digit1, multispace0, none_of},
    combinator::{map, opt, recognize, value},
    IResult,
    multi::{many0, many1},
    sequence::{delimited, pair, preceded, tuple},
};
use crate::parser::grammar::parse;

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
	Intrinsic(String), // name!!
	Comment(String),
}

pub fn tokenize(input: &str) -> IResult<&str, Vec<Token>> {
	many0(delimited(
		multispace0,
		alt((
			alt((
				map(parse_namespace, Token::Namespace),
				map(parse_variable, Token::Variable),
				value(Token::Assign, tag(":=")),
				value(Token::OpenBracket, char('[')),
				value(Token::CloseBracket, char(']')),
				value(Token::OpenParen, char('(')),
				value(Token::CloseParen, char(')')),
				value(Token::Comma, char(',')),
			)),
			map(parse_comment, |comment| Token::Comment(comment.to_string())),
			map(parse_intrinsic, Token::Intrinsic),
			map(parse_string, Token::String),
			map(parse_binary, Token::Binary),
			map(parse_hexadecimal, Token::Hexadecimal),
			map(parse_float, Token::Float),
			map(parse_integer, Token::Integer),
			map(parse_boolean, Token::Boolean),
			value(Token::Nil, tag("Nil")),
			value(Token::Pointer, tag("->")),
			map(parse_reference, Token::Reference),
			value(Token::Range, tag("..")),
			value(Token::Dot, char('.')),
			value(Token::Colon, char(':')),
			map(parse_identifier, Token::Identifier),
		)),
		multispace0,
	))(input)
}

fn parse_comment(input: &str) -> IResult<&str, String> {
	alt((
		map(
			preceded(char('#'), take_until("\n")),
			|content: &str| content.to_string()
		),
	))(input)
}

pub fn parse_namespace(input: &str) -> IResult<&str, String> {
	preceded(char('@'), parse_identifier)(input)
}

pub fn parse_intrinsic(input: &str) -> IResult<&str, String> {
	recognize(
		pair(
			parse_identifier,
			tag("!!")
		)
	)(input)
		.map(|(i, s)| (i, s.trim_end_matches("!!").to_string()))
}

pub fn parse_variable(input: &str) -> IResult<&str, String> {
	preceded(char('$'), parse_identifier)(input)
}

pub fn parse_string(input: &str) -> IResult<&str, String> {
	alt((
		delimited(
			char('\''),
			map(many0(none_of("'")), |chars: Vec<char>| {
				chars.into_iter().collect()
			}),
			char('\''),
		),
		delimited(
			char('"'),
			map(many0(none_of("\"")), |chars: Vec<char>| {
				chars.into_iter().collect()
			}),
			char('"'),
		),
	))(input)
}

pub fn parse_integer(input: &str) -> IResult<&str, i64> {
	map(recognize(pair(opt(char('-')), digit1)), |s: &str| {
		s.parse().unwrap()
	})(input)
}

pub fn parse_float(input: &str) -> IResult<&str, f64> {
	map(
		recognize(tuple((opt(char('-')), digit1, char('.'), digit1))),
		|s: &str| s.parse().unwrap(),
	)(input)
}

pub fn parse_binary(input: &str) -> IResult<&str, i64> {
	map(
		preceded(tag("0b"), take_while1(|c| c == '0' || c == '1')),
		|s: &str| i64::from_str_radix(s, 2).unwrap(),
	)(input)
}

pub fn parse_hexadecimal(input: &str) -> IResult<&str, i64> {
	map(
		preceded(tag("0x"), take_while1(|c: char| c.is_digit(16))),
		|s: &str| i64::from_str_radix(s, 16).unwrap(),
	)(input)
}

pub fn parse_boolean(input: &str) -> IResult<&str, bool> {
	alt((value(true, tag("True")), value(false, tag("False"))))(input)
}

pub fn parse_reference(input: &str) -> IResult<&str, String> {
	recognize(pair(
		alt((char('&'), char('%'))),
		many1(alt((
			alphanumeric1,
			tag("_"),
			tag("."),
			delimited(
				tag("->"),
				take_while1(|c: char| c.is_alphanumeric() || c == '_'),
				opt(delimited(char('('), take_until(")"), char(')'))),
			),
			delimited(char('['), take_until("]"), char(']')),
		))),
	))(input)
		.map(|(i, s)| (i, s.to_string()))
}

pub fn parse_identifier(input: &str) -> IResult<&str, String> {
	recognize(pair(
		alt((alpha1, tag("_"))),
		many0(alt((alphanumeric1, tag("_")))),
	))(input)
		.map(|(i, s)| (i, s.to_string()))
}
