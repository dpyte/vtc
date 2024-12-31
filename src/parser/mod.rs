use crate::parser::lexer::tokenize;
use crate::runtime::error::RuntimeError;
use crate::value::VtcFile;

pub mod ast;
pub mod grammar;
pub mod lexer;

pub fn parse_vtc(input: &str) -> Result<VtcFile, RuntimeError> {
	let (remaining, tokens) = tokenize(input)
		.map_err(|e| RuntimeError::ParseError(format!("Tokenization failed: {:?}", e)))?;
	if !remaining.is_empty() {
		return Err(RuntimeError::ParseError(
			"Input was not fully parsed".to_string(),
		));
	}
	grammar::parse(&tokens).map_err(|e| RuntimeError::ParseError(e.to_string()))
}
