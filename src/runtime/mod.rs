use ::std::collections::{HashMap, HashSet};
use ::std::fs;
use ::std::path::PathBuf;
use ::std::rc::Rc;

use smallvec::SmallVec;

use crate::parser::parse_vtc;
use crate::runtime::error::RuntimeError;
use crate::runtime::std::StdLibLoader;
use crate::value::{Accessor, Reference, ReferenceType, Value, VtcFile};

pub mod runtime;
pub mod error;
pub mod std;
pub mod serialize;
mod memory;
mod utils;

/// A struct representing the runtime environment of a software program.
#[derive(Debug)]
pub struct Runtime {
	pub namespaces: HashMap<Rc<String>, HashMap<Rc<String>, Rc<Value>>>,
	std_lib_loader: StdLibLoader
}

impl Runtime {
	pub fn new() -> Self {
		Runtime {
			namespaces: HashMap::new(),
			std_lib_loader: StdLibLoader::new(),
		}
	}

	pub fn from(path: PathBuf) -> Result<Self, RuntimeError> {
		let mut rt = Self::new();
		rt.load_file(path)?;
		Ok(rt)
	}

	// File loading methods
	pub fn load_file(&mut self, path: PathBuf) -> Result<(), RuntimeError> {
		let contents = fs::read_to_string(&path)
			.map_err(|_| RuntimeError::FileReadError(path.to_str().unwrap().to_string()))?;
		self.load_vtc(&contents)
	}

	pub fn load_vtc(&mut self, input: &str) -> Result<(), RuntimeError> {
		let vtc_file = parse_vtc(input)?;
		self.load_vtc_file(vtc_file)
	}

	pub fn update_library_loader(&mut self, lib_loader: StdLibLoader) -> Result<(), RuntimeError> {
		self.std_lib_loader = lib_loader;
		Ok(())
	}

	// Parsing methods

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

	// Value retrieval methods
	pub fn get_value(&self, namespace: &str, variable: &str, accessors: &[Accessor]) -> Result<Rc<Value>, RuntimeError> {
		let reference = Reference {
			ref_type: ReferenceType::Local,
			namespace: Some(Rc::new(namespace.to_string())),
			variable: Rc::new(variable.to_string()),
			accessors: SmallVec::from(accessors.to_vec()),
		};
		self.resolve_reference(&reference)
	}

	// Helper method for type-specific value retrieval
	fn get_typed_value<T, F>(&self, namespace: &str, variable: &str, f: F) -> Result<T, RuntimeError>
	where
		F: FnOnce(&Rc<Value>) -> Result<T, RuntimeError>,
	{
		let value = self.get_value(namespace, variable, &[])?;
		f(&value)
	}

	// Runtime exploration methods
	pub fn list_namespaces(&self) -> Vec<&Rc<String>> {
		self.namespaces.keys().collect()
	}

	pub fn list_variables(&self, namespace: &str) -> Result<Vec<&Rc<String>>, RuntimeError> {
		self.namespaces.get(&Rc::new(namespace.to_string()))
			.map(|vars| vars.keys().collect())
			.ok_or_else(|| RuntimeError::NamespaceNotFound(namespace.to_string()))
	}

	// Reference resolution methods
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
				if let Some(Value::Intrinsic(_)) = items.get(0).map(|v| &**v) {
					self.resolve_intrinsics(value, visited)
				} else {
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
					if let Some(func) = self.std_lib_loader.get_function(name) {
						let args = self.collect_intrinsic_args(items, visited)?;
						Ok(func(args))
					} else {
						Err(RuntimeError::UnknownIntrinsic((**name).clone()))
					}
				} else {
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
