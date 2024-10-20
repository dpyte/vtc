use fmt::Display;
use std::error::Error;
use std::fmt;

/// Represents all possible runtime errors.
#[derive(Debug)]
pub enum RuntimeError {
	CircularReference,
	IndexOutOfBounds(usize),
	InvalidAccessor(String),
	InvalidRange(usize, usize),
	MissingNamespace,
	NoNamespaces,
	ParseError(String),
	FileReadError(String),
	TypeError(String),
	UnknownIntrinsic(String),
	InvalidIntrinsicArgs,
	ConversionError(String),
	NamespaceNotFound(String),
	VariableNotFound(String),
	NamespaceAlreadyExists(String),
}

impl Display for RuntimeError {
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
			RuntimeError::NoNamespaces => write!(f, "No namespaces found"),
			RuntimeError::InvalidIntrinsicArgs => write!(f, "Invalid number of intrinsic arguments"),
			RuntimeError::NamespaceAlreadyExists(name) => write!(f, "Namespace already exists: {}", name),
		}
	}
}

impl Error for RuntimeError {}