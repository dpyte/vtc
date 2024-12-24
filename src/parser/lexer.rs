use nom::{
	branch::alt,
	bytes::complete::{tag, take_until, take_while1},
	character::complete::{alpha1, alphanumeric1, char, digit1, multispace0, none_of},
	combinator::{map, opt, recognize, value},
	error::{ErrorKind, ParseError},
	multi::{many0, many1},
	sequence::{delimited, pair, preceded, tuple},
	IResult, Parser,
};
use smallvec::SmallVec;
use std::sync::Arc;

pub const INLINE_CAPACITY: usize = 16;

#[derive(Debug, PartialEq, Clone)]
pub enum Token {
	Namespace(Arc<String>),
	Variable(Arc<String>),
	Assign,
	OpenBracket,
	CloseBracket,
	OpenParen,
	CloseParen,
	Comma,
	String(Arc<String>),
	Integer(i64),
	Float(f64),
	Binary(i64),
	Hexadecimal(i64),
	Boolean(bool),
	Nil,
	Reference(Arc<String>),
	Pointer,
	Dot,
	Range,
	Identifier(Arc<String>),
	Colon,
	Intrinsic(Arc<String>),
	Comment(Arc<String>),
}

type TokenVec = SmallVec<[Token; INLINE_CAPACITY]>;

#[derive(Debug)]
pub enum VtcError<I> {
	Nom(I, ErrorKind),
	Parse(String),
}

impl<I> ParseError<I> for VtcError<I> {
	fn from_error_kind(input: I, kind: ErrorKind) -> Self {
		VtcError::Nom(input, kind)
	}

	fn append(_: I, _: ErrorKind, other: Self) -> Self {
		other
	}
}

fn many0_smallvec<F, I, O, E>(mut f: F) -> impl FnMut(I) -> IResult<I, SmallVec<[O; INLINE_CAPACITY]>, E>
where
	F: Parser<I, O, E>,
	I: Clone,
	E: ParseError<I>,
{
	move |mut input: I| {
		let mut result = SmallVec::with_capacity(INLINE_CAPACITY);
		loop {
			match f.parse(input.clone()) {
				Ok((i, o)) => {
					result.push(o);
					input = i;
				}
				Err(nom::Err::Error(_)) => return Ok((input, result)),
				Err(e) => return Err(e),
			}
		}
	}
}

pub fn tokenize(input: &str) -> IResult<&str, TokenVec, VtcError<&str>> {
	many0_smallvec(delimited(
		multispace0,
		alt((parse_simple_tokens, parse_complex_tokens)),
		multispace0,
	))(input)
}

fn parse_simple_tokens(input: &str) -> IResult<&str, Token, VtcError<&str>> {
	alt((
		value(Token::Assign, tag(":=")),
		value(Token::OpenBracket, char('[')),
		value(Token::CloseBracket, char(']')),
		value(Token::OpenParen, char('(')),
		value(Token::CloseParen, char(')')),
		value(Token::Comma, char(',')),
		value(Token::Pointer, tag("->")),
		value(Token::Range, tag("..")),
		value(Token::Dot, char('.')),
		value(Token::Colon, char(':')),
		value(Token::Nil, tag("Nil")),
	))(input)
}

fn parse_complex_tokens(input: &str) -> IResult<&str, Token, VtcError<&str>> {
	alt((
		map(parse_namespace, |s| Token::Namespace(Arc::new(s))),
		map(parse_variable, |s| Token::Variable(Arc::new(s))),
		map(parse_comment, |s| Token::Comment(Arc::new(s))),
		map(parse_intrinsic, |s| Token::Intrinsic(Arc::new(s))),
		map(parse_string, |s| Token::String(Arc::new(s))),
		map(parse_binary, Token::Binary),
		map(parse_hexadecimal, Token::Hexadecimal),
		map(parse_float, Token::Float),
		map(parse_integer, Token::Integer),
		map(parse_boolean, Token::Boolean),
		map(parse_reference, |s| Token::Reference(Arc::new(s))),
		map(parse_identifier, |s| Token::Identifier(Arc::new(s))),
	))(input)
}

fn parse_comment(input: &str) -> IResult<&str, String, VtcError<&str>> {
	preceded(char('#'), take_until("\n"))(input)
		.map(|(i, s)| (i, s.to_string()))
}

pub fn parse_namespace(input: &str) -> IResult<&str, String, VtcError<&str>> {
	preceded(char('@'), parse_identifier)(input)
}

pub fn parse_intrinsic(input: &str) -> IResult<&str, String, VtcError<&str>> {
	recognize(pair(parse_identifier, tag("!!")))(input)
		.map(|(i, s)| (i, s.trim_end_matches("!!").to_string()))
}

pub fn parse_variable(input: &str) -> IResult<&str, String, VtcError<&str>> {
	preceded(char('$'), parse_identifier)(input)
}

pub fn parse_string(input: &str) -> IResult<&str, String, VtcError<&str>> {
	alt((parse_single_quoted_string, parse_double_quoted_string))(input)
}

fn parse_single_quoted_string(input: &str) -> IResult<&str, String, VtcError<&str>> {
	delimited(
		char('\''),
		map(many0(none_of("'\\")), |chars: Vec<char>| {
			let mut s = String::with_capacity(chars.len());
			s.extend(chars);
			s
		}),
		char('\''),
	)(input)
}

fn parse_double_quoted_string(input: &str) -> IResult<&str, String, VtcError<&str>> {
	delimited(
		char('"'),
		map(many0(none_of("\"\\")), |chars: Vec<char>| {
			let mut s = String::with_capacity(chars.len());
			s.extend(chars);
			s
		}),
		char('"'),
	)(input)
}

pub fn parse_integer(input: &str) -> IResult<&str, i64, VtcError<&str>> {
	map(recognize(pair(opt(char('-')), digit1)), |s: &str| {
		s.parse().unwrap_or(0)
	})(input)
}

pub fn parse_float(input: &str) -> IResult<&str, f64, VtcError<&str>> {
	map(
		recognize(tuple((opt(char('-')), digit1, char('.'), digit1))),
		|s: &str| s.parse().unwrap_or(0.0),
	)(input)
}

pub fn parse_binary(input: &str) -> IResult<&str, i64, VtcError<&str>> {
	preceded(
		tag("0b"),
		map(take_while1(|c| c == '0' || c == '1'), |s: &str| {
			i64::from_str_radix(s, 2).unwrap_or(0)
		}),
	)(input)
}

pub fn parse_hexadecimal(input: &str) -> IResult<&str, i64, VtcError<&str>> {
	preceded(
		tag("0x"),
		map(take_while1(|c: char| c.is_digit(16)), |s: &str| {
			i64::from_str_radix(s, 16).unwrap_or(0)
		}),
	)(input)
}

pub fn parse_boolean(input: &str) -> IResult<&str, bool, VtcError<&str>> {
	alt((value(true, tag("True")), value(false, tag("False"))))(input)
}

pub fn parse_reference(input: &str) -> IResult<&str, String, VtcError<&str>> {
	recognize(pair(
		alt((char('&'), char('%'))),
		many1(alt((
			alphanumeric1,
			tag("_"),
			tag("."),
			parse_reference_accessor,
		))),
	))(input)
		.map(|(i, s)| (i, s.to_string()))
}

fn parse_reference_accessor(input: &str) -> IResult<&str, &str, VtcError<&str>> {
	alt((
		delimited(
			tag("->"),
			recognize(pair(
				take_while1(|c: char| c.is_alphanumeric() || c == '_'),
				opt(delimited(char('('), take_until(")"), char(')'))),
			)),
			opt(char(')')),
		),
		delimited(char('['), take_until("]"), char(']')),
	))(input)
}

pub fn parse_identifier(input: &str) -> IResult<&str, String, VtcError<&str>> {
	recognize(pair(
		alt((alpha1, tag("_"))),
		many0(alt((alphanumeric1, tag("_")))),
	))(input)
		.map(|(i, s)| (i, s.to_string()))
}