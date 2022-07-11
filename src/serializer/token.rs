use std::fmt;
use std::fs::File;
use std::fmt::Formatter;
use std::io::{Error, Read};

#[inline]
fn str_peek(buff: &String, c_idx: &usize) -> char {
	let next = match buff.chars().nth(c_idx + 1) {
		Some(value) => value,
		None => '\0'
	};
	next
}

#[inline]
fn check_for_special_chars(c: char) -> bool {
	let check = match c {
		':' | '-' | ',' | ']' | '[' | ')'
		| '(' | '{' | '}' | '.' => true,
		_   => false,
	};
	check
}

/*
	println!("Token  \t\tValue\n---------------------");
	for tok in &tokens {
		let value: String = match tok {
			TokenKind::Literal(l) => l.clone().value,
			TokenKind::Err(e) => e.clone().msg,
			_ => "".to_string(),
		};
		println!("{:?}\t\t{}", tok, value);
	}
*/

#[derive(PartialEq, Clone)]
pub enum LitKind {
	String,
	Float,
	Bool,
	Null,
	Bin,
	Hex,
	Int,
}

impl fmt::Debug for LitKind {
	fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
		let l_value = match *self {
			LitKind::String => {"String"},
			LitKind::Float  => {"Float" },
			LitKind::Bool   => {"Bool"  },
			LitKind::Null   => {"Null"  },
			LitKind::Bin    => {"Bin"   },
			LitKind::Hex    => {"Hex"   },
			LitKind::Int    => {"Int"   }
		};
		write!(f, "{}", l_value)
	}
}

#[derive(PartialEq, Clone)]
pub struct Lit {
	pub kind: LitKind,
	pub value: String
}

#[derive(PartialEq, Clone)]
pub struct TokErr {
	pub msg: String,
}

impl TokErr {
	#[inline]
	pub fn new(msg: String) -> Self { Self{ msg } }
	pub fn default() -> Self { Self { msg: "".to_string() } }
}

impl Lit {
	pub fn new(kind: LitKind, value: String) -> Self {
		Self { kind, value }
	}
}

#[derive(PartialEq, Clone)]
pub enum TokenKind {
	At,
	Amp,
	Col,
	Doll,
	ColEq,
	Comma,
	Perc,
	DbPerc,
	Dot,
	DbDot,
	TripDot,
	DashGT,
	LParen,
	RParen,
	LBrack,
	RBrack,
	LCurl,
	RCurl,
	Literal(Lit),
	Exclaim,
	Blank,
	Hash,
	Err(TokErr),
	EOF,
}

impl fmt::Debug for TokenKind {
	fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
		let w_value = match *self {
			TokenKind::At       => {"At"},
			TokenKind::Amp      => {"Amp"},
			TokenKind::Col      => {"Col"},
			TokenKind::Doll     => {"Doll"},
			TokenKind::ColEq    => {"ColEq"},
			TokenKind::Comma    => {"Comma"},
			TokenKind::Perc     => {"Perc"},
			TokenKind::DbPerc   => {"DbPerc"},
			TokenKind::Dot      => {"Dot"},
			TokenKind::DbDot    => {"DbDot"},
			TokenKind::TripDot  => {"TripDot"},
			TokenKind::DashGT   => {"DashGT"},
			TokenKind::LParen   => {"LParen"},
			TokenKind::RParen   => {"RParen"},
			TokenKind::LBrack   => {"LBrack"},
			TokenKind::RBrack   => {"RBrack"},
			TokenKind::LCurl    => {"LCurl"},
			TokenKind::RCurl    => {"RCurl"},
			TokenKind::Exclaim  => {"Exclaim"},
			TokenKind::Blank    => {"Blank"},
			TokenKind::Hash     => {"Hash"},
			TokenKind::Err(_)   => {"Err"},
			TokenKind::EOF      => {"EOF"},
			TokenKind::Literal(_) => {"Literal"},
		};
		write!(f, "{}", w_value)
	}
}

pub struct Tokens {
	file_data:  String,
	tokens:     Vec<TokenKind>,
	data:       Vec<String>
}

impl Tokens {
	///
	/// Initialize empty token list and read file data
	///
	pub fn new(filename: &str) -> Result<Self, Error> {
		let mut file_data = String::new();

		let mut file = File::open(filename)?;
		file.read_to_string(&mut file_data)?;

		let tokens = vec![];
		let data = vec![];
		Ok(Self { file_data, tokens, data })
	}

	/// Returns total size of tokens
	pub fn len(&self) -> usize {
		self.tokens.len()
	}

	/// Return tokens
	pub fn tokens(&self) -> &Vec<TokenKind> {
		&self.tokens
	}

	pub fn tokenize(&mut self) -> Result<(), Error>{
		let len = self.file_data.len();
		let data = self.file_data.clone();

		let mut err = false;
		let mut idx = 0;
		while idx != len {
			let cchar = data.chars().nth(idx).unwrap_or('\0');
			let value = match cchar {
				//// Colon[':']
				':' => {
					let nchar = str_peek(&data, &idx);
					let p_col = if nchar == '=' {
						idx += 1;
						TokenKind::ColEq
					} else {
						TokenKind::Col
					};
					p_col
				},
				'-' => {
					let nchar = str_peek(&data, &idx);
					let a_col = if nchar == '>' {
						idx += 1;
						TokenKind::DashGT
					} else {
						err = true;
						let err_block = Self::generate_err_block(&data, &idx, " Invalid token character. Perhaps you meant '->'");
						TokenKind::Err(TokErr::new(err_block))
					};
					a_col
				}
				//// N-Blocks
				'@' => {
					let (value, index) = Self::process_n_block_chars(&data, &idx, TokenKind::At);
					idx = index;
					value
				},
				'&' => {
					let (value, index) = Self::process_n_block_chars(&data, &idx, TokenKind::Amp);
					idx = index;
					value
				}
				'$' => {
					let (value, index) = Self::process_n_block_chars(&data, &idx, TokenKind::Doll);
					idx = index;
					value
				},
				//// Brackets
				'[' => TokenKind::LBrack,
				']' => TokenKind::RBrack,
				'(' => TokenKind::LParen,
				')' => TokenKind::RParen,
				'{' => TokenKind::LCurl,
				'}' => TokenKind::RCurl,
				//// !
				'!' => TokenKind::Exclaim,
				//// ,
				',' => TokenKind::Comma,
				//// Dots
				'.' => {
					let mut c_idx = idx.clone();
					while data.chars().nth(c_idx).unwrap() == '.' { c_idx += 1 }

					let count = c_idx - idx;
					let dot_count = match count {
						1 => TokenKind::Dot,
						2 => TokenKind::DbDot,
						3 => TokenKind::TripDot,
						_ => {
							err = true;
							let err_msg = Self::generate_err_block(&data, &idx, " Invalid token character. Perhaps you meant to use one of these [., .., ...]");
							TokenKind::Err(TokErr::new(err_msg))
						},
					};
					idx += count - 1;
					dot_count
				}
				//// %, %%
				'%' => {
					let nchar = str_peek(&data, &idx);
					let mut token_is: TokenKind = TokenKind::Blank;
					match nchar {
						'%' => { idx += 1;  token_is = TokenKind::DbPerc; },
						'a'..='z' | 'A'..='Z' | '_'   => { token_is = TokenKind::Perc; }
						// Pointer to a numerical is prohibited
						' ' | '0'..='9' => {
							err = true;
							let err_msg = Self::generate_err_block(&data, &idx, " Encountered illegal token after '%'");
							token_is = TokenKind::Err(TokErr::new(err_msg));
						}
						_ => {}
					};
					token_is
				}
				//// Comment block
				'#' => {
					let (skip_to, comment_block) = Self::parse_comment_block(&data, idx.clone());
					self.data.push(comment_block);
					idx = skip_to;
					TokenKind::Hash
				}
				//// Assignment Error
				'=' => {
					err = true;
					let err_block = Self::generate_err_block(&data, &idx, " Use ':=' for assignment operations");
					TokenKind::Err(TokErr::new(err_block))
				}
				//// AlphaNumeric + Misc
				_ => {
					let (token, index) = Self::process_alpha_numeric_misc(&data, &idx);
					idx = index;
					token
				},
			};

			idx += 1;
			if value == TokenKind::EOF { break }
			if err {
				let err_msg = match value.clone() {
					TokenKind::Err(e) => e.msg,
					_ => TokErr::default().msg
				};
				eprintln!("{}", err_msg);
				break
				// One way to deal with this error is to stack-dump (?)
				// TODO: Figure out the fastest way to do this ...
			}
			self.tokens.push(value);
		}

		// Cleanup
		if !err { self.tokens.retain(|r| *r != TokenKind::Blank); }
		Ok(())
	}

	///
	/// Parse comment block
	///
	#[inline]
	fn parse_comment_block(data: &String, mut idx: usize) -> (usize, String) {
		let c_idx = idx.clone();
		while data.chars().nth(idx).unwrap() != '\n' { idx += 1; }
		let skip_by = idx - c_idx;
		let comment_block: String = data.chars().skip(c_idx).take(skip_by).collect();
		(idx, comment_block)
	}

	///
	/// Create 'fancy' error block
	///
	#[inline]
	fn generate_err_block(data: &String, idx: &usize, msg: &str) -> String {
		let c_idx = idx.clone();
		let mut w_idx = idx.clone();
		while data.chars().nth(w_idx).unwrap() != '\n' {
			w_idx += 1;
		}
		let mut error: String = data.chars().skip(*idx).take(w_idx - c_idx).collect();
		error.push_str("\n^~~~");
		error.push_str(msg);
		error
	}

	///
	/// Filter out alphanumeric values
	/// TODO: Return error on failure
	///
	#[inline]
	fn process_alpha_numeric_misc(data: &String, idx: &usize) -> (TokenKind, usize) {
		let mut c_idx = idx.clone();
		let mut token = TokenKind::EOF;

		let failure = false;
		let mut value: String = String::new();
		let err: TokErr = TokErr { msg: "".to_string() };

		loop {
			let c_value = data.chars().nth(c_idx).unwrap();
			// TODO: Logic fix
			// Current logic increments the index counter raising issues in the program.
			// If it encounters these characters then it tries to decrement the counter by one
			if check_for_special_chars(c_value) { c_idx -= 1; }
			let is_valid = match c_value {
				'a'..='z' | 'A'..='Z' | '_' | '0'..='9' => true,
				_ => false,
			};

			if is_valid {
				c_idx += 1;
				value.push(c_value);
			} else { break; }
		}

		// Return early
		if failure { return (TokenKind::Err(err), c_idx) }
		if value.is_empty() { return (TokenKind::Blank, c_idx) }

		if value.chars().all(char::is_alphanumeric) || !value.is_empty() {
			// Integer Check
			let lit_check = match value.parse::<i64>() {
				Ok(_) => LitKind::Float,
				Err(_) => LitKind::String
			};
			let lit_kind = Lit::new(lit_check, value);
			token = TokenKind::Literal(lit_kind);
		}
		(token, c_idx)
	}

	///
	/// n block characters include @, &, and $
	///
	#[inline]
	fn process_n_block_chars(data: &String, idx: &usize, r_type: TokenKind) -> (TokenKind, usize) {
		let w_idx = idx.clone();
		let nchar = str_peek(&data, &w_idx);

		let retval = match nchar {
			' ' => {
				let err_block = Self::generate_err_block(&data, &w_idx, " Expects [a-zA-Z0-0] after '[@, &, $]'");
				TokenKind::Err(TokErr::new(err_block))
			},
			_   => { r_type }
		};
		(retval, w_idx)
	}
}
