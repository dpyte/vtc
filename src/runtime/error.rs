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
}
