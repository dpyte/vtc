use std::ffi::c_void;
use std::fmt;
use std::fmt::Formatter;
use std::process::id;
use crate::serializer::token::LitKind;
use crate::serializer::types::{Types, ValType};
use crate::serializer::token::TokenKind::Literal;
use crate::serializer::token::{TokenKind, Tokens};
use crate::Stack;

#[derive(Debug)]
struct Tag {
	pub t_value_1: String,
	pub t_value_2: String,
}

impl Tag {
	pub fn default() -> Self {
		Self {
			t_value_1: "".to_string(),
			t_value_2: "".to_string(),
		}
	}
}

#[derive(Debug)]
struct Reference {
	pub to_ref_value: String,
	pub reference_range: Vec<u16>
}

#[derive(Debug)]
struct Pointer {
	pub pointing_container: String,
	pub pointing_value: String,
	pub reference_range: Vec<u16>
}

/// ListType stores the bits and segments of the value such as
/// Reference, Pointer, or by Value
/// * store_type: Type of value stored within
/// * val_type: Type of value i.e., string, float, integer, ...
/// * ref_to: An optional field that defines what this value references to, if applicable
/// * points_to: An optional field that defines what this value points to, if applicable
#[derive(Debug)]
struct ListType {
	pub store_type: ValType,
	pub val_type: Types,
	pub value: String,
	pub ref_to: Option<Reference>,
	pub points_to: Option<Pointer>,
}

/// Annotate and store type of value a PValue field may contain
/// All values inside the field has to be a list
#[derive(Debug)]
enum VarType {
	EmptyList(String),
	List(Vec<ListType>)
}

/// This structure is internal to parser and should not be conflicted with
/// the `container` structure found in container.rs
/// * c_name: Name of the container
/// * values: Intermediate representation of the values
///     - First field is for name of the variable
///     - Second field is for the contained value
struct PContainer {
	pub c_name: String,
	pub values: Vec<(String, VarType)>
}

impl PContainer {
	/// Initialize empty container
	#[inline]
	pub fn default() -> Self {
		Self { c_name: String::new(), values: vec![] }
	}

	pub fn update_name(&mut self, name: &String) { self.c_name = name.clone(); }
}

impl fmt::Debug for PContainer {
	fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
		f.debug_struct("PContainer")
			.field("c_name", &self.c_name)
			.field("values", &self.values)
			.finish()
	}
}

pub struct RParser {
	tag: Vec<Tag>,
	p_container: Vec<PContainer>,
	tokens: Tokens,
	cursor: usize,
}

impl RParser {
	/// Constructs a new Root parser and populates with the tokens
	pub fn new(tokens: Tokens) -> Self {
		Self { tag: vec![], p_container: vec![], tokens, cursor: 0, }
	}

	/// Generate a simple-AST
	pub fn generate_ast(&mut self) {
		let total_token_count = self.tokens.tokens().len();
		let tokens = self.tokens.tokens();

		if tokens.is_empty() { return; }

		let mut tags: Vec<Tag> = vec![];
		loop {
			let c_tok = tokens.iter().nth(self.cursor).unwrap();

			// Throw an error if starting token is not DbPerc | At
			let index = match c_tok {
				TokenKind::DbPerc => {
					let (tag, idx) = Self::parse_tags(&tokens, &self.cursor);
					tags.push(tag);
					idx
				},
				TokenKind::At => {
					let (cont, idx) = Self::container_begin(&tokens, &self.cursor);
					idx
				},
				TokenKind::Hash => (self.cursor + 1).try_into().unwrap(),
				_ => -1,
			};

			if index <= 0 {
				// TODO: raise error
				println!("Encountered issue parsing: {:?}", c_tok);
				break
			}
			self.cursor += index as usize;

			self.cursor += 1;
			if self.cursor >= total_token_count { break }
		}
	}

	/// Peek through the next token value
	#[inline]
	fn peek<'a>(tokens: &'a Vec<TokenKind>, c_idx: &'a usize) -> &'a TokenKind {
		let n_tok = tokens.iter().nth(c_idx + 1);
		match n_tok {
			Some(v) => v,
			None => &TokenKind::Blank
		}
	}

	/// Parse container:
	/// @container: ...
	/// Grammar: <@> + <String> + <:>
	///     + <$> + <String> + <:=> + <[> + <ListValue> + <]>
	#[inline]
	fn container_begin(tokens: &Vec<TokenKind>, c_idx: &usize) -> (PContainer, i32) {
		// We know that current index points to TokenKind::At
		let mut w_idx = c_idx + 1;
		let t_size = tokens.len();
		// Return error if index at next token == total size of token_kind
		if w_idx >= t_size { return (PContainer::default(), -1) }
		let mut t_container = PContainer::default();

		// Extract container name:
		let pk = Self::peek(&tokens, &w_idx);
		let container_name = match pk {
			TokenKind::Literal(v) => v.value.clone(),
			_ => String::new(),
		};
		println!("Container name: {}", container_name);
		if container_name.is_empty() { return (t_container, -1) }
		t_container.c_name = container_name;

		w_idx += 1;
		loop {
			let w_tok = tokens.iter().nth(w_idx).unwrap();
			match w_tok {
				_ => {}
			};

			w_idx += 1;
			if w_idx >= t_size { break }
		}

		println!("{:#?}", t_container);
		(t_container, w_idx as i32)
	}

	/// Parse tags:
	/// %% foo bar ...
	/// Grammar: <%%> + <String> + <String>
	#[inline]
	fn parse_tags(tokens: &Vec<TokenKind>, c_idx: &usize) -> (Tag, i32) {
		let w_idx = c_idx + 1;
		let in_range = c_idx + 2 <= tokens.len();

		// Early error
		if !in_range { return (Tag::default(), -1) }

		let mut lit_count = 0;
		let mut tag_values = Vec::new();
		loop {
			let w_tok = tokens.iter().nth(w_idx).unwrap();
			let value = match w_tok {
				Literal(v) => v.value.clone(),
				_ => "".to_string(),
			};
			// Failed first check
			if value.is_empty() { return(Tag::default(), -1) }
			println!("Tag_x :: {}", value);
			tag_values.push(value);

			lit_count += 1;
			if lit_count == 2 { break }
		}
		assert_eq!(tag_values.len(), 2);

		let tag = Tag {
			t_value_1: tag_values.iter().nth(0).unwrap().clone(),
			t_value_2: tag_values.iter().nth(1).unwrap().clone(),
		};

		(tag, (w_idx + 2) as i32)
	}
}
