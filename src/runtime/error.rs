use std::fmt;

/// Represents all possible runtime errors.
#[derive(Debug)]
pub enum RuntimeError {
	CircularReference,
	IndexOutOfBounds(usize),
	InvalidAccessor(String),
	InvalidRange(usize, usize),
	MissingNamespace,
	NamespaceNotFound(String),
	NoNamespaces,
	ParseError(String),
	VariableNotFound(String),
	FileReadError(String),
	TypeError(String),
	UnknownIntrinsic(String),
	InvalidIntrinsicArgs,
	ConversionError(String),
}

impl fmt::Display for RuntimeError {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		match self {
			RuntimeError::FileReadError(path) => write!(f, "Failed to read file: {}", path),
			RuntimeError::ParseError(msg) => write!(f, "Parse error: {}", msg),
			RuntimeError::NamespaceNotFound(name) => write!(f, "Namespace not found: {}", name),
			RuntimeError::VariableNotFound(name) => write!(f, "Variable not found: {}", name),
			RuntimeError::CircularReference => write!(f, "Circular reference detected"),
			RuntimeError::MissingNamespace => write!(f, "Missing namespace"),
			RuntimeError::IndexOutOfBounds(index) => write!(f, "Index out of bounds: {}", index),
			RuntimeError::InvalidRange(start, end) => write!(f, "Invalid range: {} to {}", start, end),
			RuntimeError::InvalidAccessor(msg) => write!(f, "Invalid accessor: {}", msg),
			RuntimeError::TypeError(msg) => write!(f, "Type error: {}", msg),
			RuntimeError::ConversionError(msg) => write!(f, "Conversion error: {}", msg),
			RuntimeError::UnknownIntrinsic(name) => write!(f, "Unknown intrinsic: {}", name),
			RuntimeError::NoNamespaces => {write!(f, "Namespace not found")},
			RuntimeError::InvalidIntrinsicArgs => {write!(f, "Invalid amount of intrinsic args")}
		}
	}
}