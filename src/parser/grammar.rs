use crate::parser::ast::{Accessor, Namespace, Number, Reference, ReferenceType, Value, Variable, VtcFile};
use crate::parser::lexer::Token;
use nom::branch::alt;
use nom::combinator::map;
use nom::multi::{many0, many1, separated_list0};
use nom::sequence::{delimited, preceded};
use nom::IResult;

pub fn parse(tokens: &[Token]) -> IResult<&[Token], VtcFile> {
    map(
        many1(parse_namespace),
        |namespaces| VtcFile { namespaces }
    )(tokens)
}

pub fn parse_namespace(tokens: &[Token]) -> IResult<&[Token], Namespace> {
    let (tokens, name) = match tokens.get(0) {
        Some(Token::Namespace(name)) => (&tokens[1..], name.clone()),
        _ => return Err(nom::Err::Error(nom::error::Error::new(tokens, nom::error::ErrorKind::Tag))),
    };

    let (tokens, _) = match tokens.get(0) {
        Some(Token::Colon) => (&tokens[1..], ()),
        _ => return Err(nom::Err::Error(nom::error::Error::new(tokens, nom::error::ErrorKind::Tag))),
    };

    let (tokens, variables) = many0(parse_variable)(tokens)?;

    Ok((tokens, Namespace { name, variables }))
}

fn parse_namespace_name(tokens: &[Token]) -> IResult<&[Token], String> {
    match tokens.get(0) {
        Some(Token::Namespace(name)) => Ok((&tokens[1..], name.clone())),
        _ => Err(nom::Err::Error(nom::error::Error::new(tokens, nom::error::ErrorKind::Tag))),
    }
}

fn parse_colon(tokens: &[Token]) -> IResult<&[Token], ()> {
    match tokens.get(0) {
        Some(Token::Colon) => Ok((&tokens[1..], ())),
        _ => Err(nom::Err::Error(nom::error::Error::new(tokens, nom::error::ErrorKind::Tag))),
    }
}

pub fn parse_variable(tokens: &[Token]) -> IResult<&[Token], Variable> {
    let (tokens, name) = parse_variable_name(tokens)?;
    let (tokens, _) = parse_assign(tokens)?;
    let (tokens, value) = parse_value(tokens)?;

    Ok((tokens, Variable { name, value }))
}

fn parse_variable_name(tokens: &[Token]) -> IResult<&[Token], String> {
    match tokens.get(0) {
        Some(Token::Variable(name)) => Ok((&tokens[1..], name.clone())),
        _ => Err(nom::Err::Error(nom::error::Error::new(tokens, nom::error::ErrorKind::Tag))),
    }
}

fn parse_assign(tokens: &[Token]) -> IResult<&[Token], ()> {
    match tokens.get(0) {
        Some(Token::Assign) => Ok((&tokens[1..], ())),
        _ => Err(nom::Err::Error(nom::error::Error::new(tokens, nom::error::ErrorKind::Tag))),
    }
}

pub fn parse_value<'a>(tokens: &'a [Token]) -> IResult<&'a [Token], Value> {
    alt((
        map(parse_string, Value::String),
        map(parse_number, Value::Number),
        map(parse_boolean, Value::Boolean),
        map(parse_nil, |_| Value::Nil),
        map(parse_list, Value::List),
        map(parse_reference, Value::Reference),
    ))(tokens)
}

fn parse_identifier(tokens: &[Token]) -> IResult<&[Token], String> {
    match tokens.get(0) {
        Some(Token::Identifier(s)) => Ok((&tokens[1..], s.clone())),
        _ => Err(nom::Err::Error(nom::error::Error::new(tokens, nom::error::ErrorKind::Tag))),
    }
}

fn parse_string(tokens: &[Token]) -> IResult<&[Token], String> {
    match tokens.get(0) {
        Some(Token::String(s)) => Ok((&tokens[1..], s.clone())),
        _ => Err(nom::Err::Error(nom::error::Error::new(tokens, nom::error::ErrorKind::Tag))),
    }
}

fn parse_boolean(tokens: &[Token]) -> IResult<&[Token], bool> {
    match tokens.get(0) {
        Some(Token::Boolean(b)) => Ok((&tokens[1..], *b)),
        _ => Err(nom::Err::Error(nom::error::Error::new(tokens, nom::error::ErrorKind::Tag))),
    }
}

fn parse_nil(tokens: &[Token]) -> IResult<&[Token], ()> {
    match tokens.get(0) {
        Some(Token::Nil) => Ok((&tokens[1..], ())),
        _ => Err(nom::Err::Error(nom::error::Error::new(tokens, nom::error::ErrorKind::Tag))),
    }
}

pub fn parse_number(tokens: &[Token]) -> IResult<&[Token], Number> {
    match tokens.get(0) {
        Some(Token::Integer(i)) => Ok((&tokens[1..], Number::Integer(*i))),
        Some(Token::Float(f)) => Ok((&tokens[1..], Number::Float(*f))),
        Some(Token::Binary(b)) => Ok((&tokens[1..], Number::Binary(*b))),
        Some(Token::Hexadecimal(h)) => Ok((&tokens[1..], Number::Hexadecimal(*h))),
        _ => Err(nom::Err::Error(nom::error::Error::new(tokens, nom::error::ErrorKind::Tag))),
    }
}

pub fn parse_list<'a>(tokens: &'a [Token]) -> IResult<&'a [Token], Vec<Value>> {
    delimited(
        parse_open_bracket,
        separated_list0(parse_comma, parse_value),
        parse_close_bracket
    )(tokens)
}

fn parse_open_bracket(tokens: &[Token]) -> IResult<&[Token], ()> {
    match tokens.get(0) {
        Some(Token::OpenBracket) => Ok((&tokens[1..], ())),
        _ => Err(nom::Err::Error(nom::error::Error::new(tokens, nom::error::ErrorKind::Tag))),
    }
}

fn parse_close_bracket(tokens: &[Token]) -> IResult<&[Token], ()> {
    match tokens.get(0) {
        Some(Token::CloseBracket) => Ok((&tokens[1..], ())),
        _ => Err(nom::Err::Error(nom::error::Error::new(tokens, nom::error::ErrorKind::Tag))),
    }
}

fn parse_comma(tokens: &[Token]) -> IResult<&[Token], ()> {
    match tokens.get(0) {
        Some(Token::Comma) => Ok((&tokens[1..], ())),
        _ => Err(nom::Err::Error(nom::error::Error::new(tokens, nom::error::ErrorKind::Tag))),
    }
}

pub fn parse_reference(tokens: &[Token]) -> IResult<&[Token], Reference> {
    let (tokens, reference_token) = match tokens.get(0) {
        Some(Token::Reference(r)) => (&tokens[1..], r),
        _ => return Err(nom::Err::Error(nom::error::Error::new(tokens, nom::error::ErrorKind::Tag))),
    };

    let (ref_type, rest) = if reference_token.starts_with('&') {
        (ReferenceType::External, &reference_token[1..])
    } else if reference_token.starts_with('%') {
        (ReferenceType::Local, &reference_token[1..])
    } else {
        return Err(nom::Err::Error(nom::error::Error::new(tokens, nom::error::ErrorKind::Tag)));
    };

    let (namespace, variable, rest) = if let Some(dot_index) = rest.find('.') {
        let namespace = Some(rest[..dot_index].to_string());
        let rest = &rest[dot_index + 1..];
        if let Some(var_end) = rest.find(|c| c == '-' || c == '[') {
            let variable = rest[..var_end].to_string();
            (namespace, variable, &rest[var_end..])
        } else {
            (namespace, rest.to_string(), "")
        }
    } else if let Some(var_end) = rest.find(|c| c == '-' || c == '[') {
        let variable = rest[..var_end].to_string();
        (None, variable, &rest[var_end..])
    } else {
        (None, rest.to_string(), "")
    };

    let (tokens, accessors) = parse_accessors(tokens)?;

    Ok((tokens, Reference {
        ref_type,
        namespace,
        variable,
        accessors,
    }))
}

fn parse_ref_type(r: &str) -> (ReferenceType, &str) {
    if r.starts_with('&') {
        (ReferenceType::External, &r[1..])
    } else if r.starts_with('%') {
        (ReferenceType::Local, &r[1..])
    } else {
        panic!("Invalid reference type")
    }
}

fn parse_namespace_and_variable(s: &str) -> (Option<String>, String, &str) {
    if let Some(dot_index) = s.find('.') {
        let namespace = Some(s[..dot_index].to_string());
        let rest = &s[dot_index + 1..];
        if let Some(var_end) = rest.find(|c| c == '-' || c == '[') {
            let variable = rest[..var_end].to_string();
            (namespace, variable, &rest[var_end..])
        } else {
            (namespace, rest.to_string(), "")
        }
    } else if let Some(var_end) = s.find(|c| c == '-' || c == '[') {
        let variable = s[..var_end].to_string();
        (None, variable, &s[var_end..])
    } else {
        (None, s.to_string(), "")
    }
}

fn parse_accessors(tokens: &[Token]) -> IResult<&[Token], Vec<Accessor>> {
    many0(
        alt((
            preceded(parse_pointer, parse_index_accessor),
            preceded(parse_pointer, parse_key_accessor),
        ))
    )(tokens)
}

fn parse_index_accessor(tokens: &[Token]) -> IResult<&[Token], Accessor> {
    delimited(
        parse_open_paren,
        map(parse_integer, |i| Accessor::Index(i as usize)),
        parse_close_paren
    )(tokens)
}

fn parse_key_accessor(tokens: &[Token]) -> IResult<&[Token], Accessor> {
    delimited(
        parse_open_bracket,
        map(parse_identifier, Accessor::Key),
        parse_close_bracket
    )(tokens)
}


fn parse_open_paren(tokens: &[Token]) -> IResult<&[Token], ()> {
    match tokens.get(0) {
        Some(Token::OpenParen) => Ok((&tokens[1..], ())),
        _ => Err(nom::Err::Error(nom::error::Error::new(tokens, nom::error::ErrorKind::Tag))),
    }
}

fn parse_close_paren(tokens: &[Token]) -> IResult<&[Token], ()> {
    match tokens.get(0) {
        Some(Token::CloseParen) => Ok((&tokens[1..], ())),
        _ => Err(nom::Err::Error(nom::error::Error::new(tokens, nom::error::ErrorKind::Tag))),
    }
}

fn parse_pointer(tokens: &[Token]) -> IResult<&[Token], ()> {
    match tokens.get(0) {
        Some(Token::Pointer) => Ok((&tokens[1..], ())),
        _ => Err(nom::Err::Error(nom::error::Error::new(tokens, nom::error::ErrorKind::Tag))),
    }
}

fn parse_integer(tokens: &[Token]) -> IResult<&[Token], i64> {
    match tokens.get(0) {
        Some(Token::Integer(i)) => Ok((&tokens[1..], *i)),
        _ => Err(nom::Err::Error(nom::error::Error::new(tokens, nom::error::ErrorKind::Tag))),
    }
}