use std::collections::{HashMap, HashSet};
use std::fs;
use std::hash::Hash;
use std::path::PathBuf;
use std::rc::Rc;

use smallvec::SmallVec;

use crate::parser::grammar::parse;
use crate::parser::lexer::tokenize;
use crate::runtime::error::RuntimeError;
use crate::{Accessor, Number, Reference, ReferenceType, Value, VtcFile};
use crate::runtime::std::StdLibLoader;

/// Represents the runtime environment for VTC files.
#[derive(Debug)]
pub struct Runtime {
	namespaces: HashMap<Rc<String>, HashMap<Rc<String>, Rc<Value>>>,
}

impl Runtime {
	/// Creates a new `Runtime` instance.
	pub fn new() -> Self {
		Runtime {
			namespaces: HashMap::new(),
		}
	}

	pub fn from(path: PathBuf) -> Result<Self, RuntimeError> {
		let mut rt = Self::new();
		rt.load_file(path)?;
		Ok(rt)
	}

	/// Reads a VTC file from the given path and loads it into the runtime.
	pub fn load_file(&mut self, path: PathBuf) -> Result<(), RuntimeError> {
		let contents = fs::read_to_string(&path)
			.map_err(|_| RuntimeError::FileReadError(path.to_str().unwrap().to_string()))?;
		self.load_vtc(&contents)
	}

	/// Loads VTC content from a string into the runtime.
	pub fn load_vtc(&mut self, input: &str) -> Result<(), RuntimeError> {
		let vtc_file = self.parse_vtc(input)?;
		self.load_vtc_file(vtc_file)
	}

	// Private methods for parsing and loading

	fn parse_vtc(&self, input: &str) -> Result<VtcFile, RuntimeError> {
		let (remaining, tokens) = tokenize(input)
			.map_err(|e| RuntimeError::ParseError(format!("Tokenization failed: {:?}", e)))?;
		if !remaining.is_empty() {
			return Err(RuntimeError::ParseError("Input was not fully parsed".to_string()));
		}
		parse(&tokens).map_err(|e| RuntimeError::ParseError(e.to_string()))
	}

	fn load_vtc_file(&mut self, vtc_file: VtcFile) -> Result<(), RuntimeError> {
		for namespace in vtc_file.namespaces {
			let variables: HashMap<Rc<String>, Rc<Value>> = namespace.variables
				.iter()
				.map(|var| (Rc::clone(&var.name), Rc::clone(&var.value)))
				.collect();
			self.namespaces.insert(Rc::clone(&namespace.name), variables);
		}
		Ok(())
	}

	// Public methods for value retrieval

	/// Retrieves a value from the specified namespace and variable.
	pub fn get_value(&self, namespace: &str, variable: &str, accessors: &[Accessor]) -> Result<Rc<Value>, RuntimeError> {
		let reference = Reference {
			ref_type: ReferenceType::Local,
			namespace: Some(Rc::new(namespace.to_string())),
			variable: Rc::new(variable.to_string()),
			accessors: SmallVec::from(accessors.to_vec()),
		};
		self.resolve_reference(&reference)
	}

	pub fn get_string_value(&self, value: &Rc<Value>) -> Result<String, RuntimeError> {
		if let Value::String(s) = &**value {
			Ok((**s).clone())
		} else {
			Err(RuntimeError::TypeError("Expected string".to_string()))
		}
	}

	/// Retrieves a string value from the specified namespace and variable.
	pub fn get_string(&self, namespace: &str, variable: &str) -> Result<String, RuntimeError> {
		let value = self.get_value(namespace, variable, &[])?;
		self.get_string_value(&value)
	}

	/// Converts a Value to its string representation.
	pub fn to_string(&self, value: Rc<Value>) -> Result<String, RuntimeError> {
		Ok(value.to_string()[1..value.to_string().len() - 1].to_string())
	}

	/// Retrieves an integer value from the specified namespace and variable.
	pub fn get_integer(&self, namespace: &str, variable: &str) -> Result<i64, RuntimeError> {
		self.get_typed_value(namespace, variable, |v| {
			if let Value::Number(Number::Integer(i)) = &**v {
				Ok(*i)
			} else {
				Err(RuntimeError::TypeError("Expected integer".to_string()))
			}
		})
	}

	/// Retrieves a float value from the specified namespace and variable.
	pub fn get_float(&self, namespace: &str, variable: &str) -> Result<f64, RuntimeError> {
		self.get_typed_value(namespace, variable, |v| {
			if let Value::Number(Number::Float(f)) = &**v {
				Ok(*f)
			} else {
				Err(RuntimeError::TypeError("Expected float".to_string()))
			}
		})
	}

	/// Retrieves a boolean value from the specified namespace and variable.
	pub fn get_boolean(&self, namespace: &str, variable: &str) -> Result<bool, RuntimeError> {
		self.get_typed_value(namespace, variable, |v| {
			if let Value::Boolean(b) = &**v {
				Ok(*b)
			} else {
				Err(RuntimeError::TypeError("Expected boolean".to_string()))
			}
		})
	}

	/// Retrieves a list value from the specified namespace and variable.
	pub fn get_list(&self, namespace: &str, variable: &str) -> Result<Vec<Value>, RuntimeError> {
		self.get_typed_value(namespace, variable, |v| {
			if let Value::List(l) = &**v {
				Ok(l
					.iter()
					.map(|value| (**value).clone())
					.collect::<Vec<_>>())
			} else {
				Err(RuntimeError::TypeError("Expected list".to_string()))
			}
		})
	}

	// Helper method for type-specific value retrieval
	fn get_typed_value<T, F>(&self, namespace: &str, variable: &str, f: F) -> Result<T, RuntimeError>
	where
		F: FnOnce(&Rc<Value>) -> Result<T, RuntimeError>,
	{
		let value = self.get_value(namespace, variable, &[])?;
		f(&value)
	}

	// Methods for exploring the runtime content

	/// Lists all namespaces in the runtime.
	pub fn list_namespaces(&self) -> Vec<&Rc<String>> {
		self.namespaces.keys().collect()
	}

	/// Lists all variables in a specific namespace.
	pub fn list_variables(&self, namespace: &str) -> Result<Vec<&Rc<String>>, RuntimeError> {
		self.namespaces.get(&Rc::new(namespace.to_string()))
			.map(|vars| vars.keys().collect())
			.ok_or_else(|| RuntimeError::NamespaceNotFound(namespace.to_string()))
	}

	// Private methods for reference resolution

	fn resolve_reference(&self, reference: &Reference) -> Result<Rc<Value>, RuntimeError> {
		let mut visited = HashSet::new();
		self.resolve_reference_recursive(reference, &mut visited)
	}

	fn resolve_reference_recursive(
		&self,
		reference: &Reference,
		visited: &mut HashSet<(Rc<String>, Rc<String>)>,
	) -> Result<Rc<Value>, RuntimeError> {
		let namespace = reference.namespace.as_ref()
			.ok_or_else(|| RuntimeError::MissingNamespace)?;
		let key = (Rc::clone(namespace), Rc::clone(&reference.variable));

		if !visited.insert(key.clone()) {
			return Err(RuntimeError::CircularReference);
		}

		let variables = self.namespaces.get(namespace)
			.ok_or_else(|| RuntimeError::NamespaceNotFound((**namespace).clone()))?;

		let mut value = variables.get(&reference.variable)
			.ok_or_else(|| RuntimeError::VariableNotFound((**reference.variable).parse().unwrap()))?
			.clone();

		value = self.resolve_value(value, visited)?;
		value = self.resolve_intrinsics(value, visited)?;

		for accessor in &reference.accessors {
			value = self.apply_accessor(value, accessor)?;
		}

		visited.remove(&key);
		Ok(value)
	}

	// Personal note1: This is going to get somewhat messy... The intrinsics will need to be carefully evaluated...
	fn resolve_value(
		&self,
		value: Rc<Value>,
		visited: &mut HashSet<(Rc<String>, Rc<String>)>,
	) -> Result<Rc<Value>, RuntimeError> {
		match &*value {
			Value::Reference(inner_reference) => {
				self.resolve_reference_recursive(inner_reference, visited)
			}
			Value::List(items) => {
				// Check if this list is an intrinsic call
				if let Some(Value::Intrinsic(_)) = items.get(0).map(|v| &**v) {
					self.resolve_intrinsics(value, visited)
				} else {
					// If not, resolve each item in the list
					let resolved_items = items
						.iter()
						.map(|item| self.resolve_value(Rc::clone(item), visited))
						.collect::<Result<Vec<_>, _>>()?;
					Ok(Rc::new(Value::List(Rc::new(resolved_items))))
				}
			}
			_ => Ok(value),
		}
	}

	pub fn resolve_intrinsics(
		&self,
		value: Rc<Value>,
		visited: &mut HashSet<(Rc<String>, Rc<String>)>,
	) -> Result<Rc<Value>, RuntimeError> {
		match &*value {
			Value::List(items) => {
				if let Some(Value::Intrinsic(name)) = items.get(0).map(|v| &**v) {
					let std_lib = StdLibLoader::new();
					if let Some(func) = std_lib.get_function(name) {
						let args = self.collect_intrinsic_args(items, visited)?;
						Ok(func(args))
					} else {
						Err(RuntimeError::UnknownIntrinsic((**name).clone()))
					}
				} else {
					// If it's not an intrinsic call, resolve each item in the list
					let resolved_items = items
						.iter()
						.map(|item| self.resolve_intrinsics(Rc::clone(item), visited))
						.collect::<Result<Vec<_>, _>>()?;
					Ok(Rc::new(Value::List(Rc::new(resolved_items))))
				}
			}
			_ => Ok(value),
		}
	}

	fn collect_intrinsic_args(
		&self,
		items: &[Rc<Value>],
		visited: &mut HashSet<(Rc<String>, Rc<String>)>,
	) -> Result<Vec<Rc<Value>>, RuntimeError> {
		// Skip the first item (the intrinsic name) and resolve the rest
		items.iter()
			.skip(1)
			.map(|item| self.resolve_value(Rc::clone(item), visited))
			.collect()
	}


	fn apply_accessor(&self, value: Rc<Value>, accessor: &Accessor) -> Result<Rc<Value>, RuntimeError> {
		match (&*value, accessor) {
			(Value::List(list), Accessor::Index(index)) => list
				.get(*index)
				.cloned()
				.ok_or(RuntimeError::IndexOutOfBounds(*index)),
			(Value::List(list), Accessor::Range(start, end)) => {
				if *start > *end || *end > list.len() {
					Err(RuntimeError::InvalidRange(*start, *end))
				} else {
					Ok(Rc::new(Value::List(Rc::new(list[*start..*end].to_vec()))))
				}
			}
			(Value::String(s), Accessor::Index(index)) => s
				.chars()
				.nth(*index)
				.map(|c| Rc::new(Value::String(Rc::new(c.to_string()))))
				.ok_or(RuntimeError::IndexOutOfBounds(*index)),
			(Value::String(s), Accessor::Range(start, end)) => {
				if *start > *end || *end > s.len() {
					Err(RuntimeError::InvalidRange(*start, *end))
				} else {
					Ok(Rc::new(Value::String(Rc::new(s[*start..*end].to_string()))))
				}
			}
			(_, Accessor::Key(key)) => Err(RuntimeError::InvalidAccessor(format!(
				"Key accessor '{}' not supported for this value type",
				key
			))),
			_ => Err(RuntimeError::InvalidAccessor(
				"Accessor not supported for this value type".to_string(),
			)),
		}
	}
}