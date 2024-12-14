use ::std::fs;
use ::std::path::PathBuf;
use ::std::sync::Arc;
use fnv::{FnvHashMap, FnvHashSet};
use smallvec::SmallVec;

use crate::parser::parse_vtc;
use crate::runtime::error::RuntimeError;
use crate::runtime::std::StdLibLoader;
use crate::value::{Accessor, Number, Reference, ReferenceType, Value, VtcFile};

pub mod runtime;
pub mod error;
pub mod std;
pub mod serialize;
mod memory;
mod utils;

/// A thread-safe runtime environment for VTC program execution
#[derive(Debug)]
pub struct Runtime {
	pub namespaces: FnvHashMap<Arc<String>, FnvHashMap<Arc<String>, Arc<Value>>>,
	std_lib_loader: StdLibLoader
}

impl Runtime {
	pub fn new() -> Self {
		Runtime {
			namespaces: FnvHashMap::default(),
			std_lib_loader: StdLibLoader::new(),
		}
	}

	pub fn from(path: PathBuf) -> Result<Self, RuntimeError> {
		let mut rt = Self::new();
		rt.load_file(path)?;
		Ok(rt)
	}

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

	fn load_vtc_file(&mut self, vtc_file: VtcFile) -> Result<(), RuntimeError> {
		for namespace in vtc_file.namespaces {
			let mut variables = FnvHashMap::with_capacity_and_hasher(
				namespace.variables.len(),
				Default::default(),
			);

			for var in namespace.variables {
				let key = Arc::new(var.name);
				let value = Arc::new(var.value);
				variables.insert(key, value);
			}

			self.namespaces.insert(Arc::new(namespace.name), variables);
		}
		Ok(())
	}

	pub fn get_value(&self, namespace: &str, variable: &str, accessors: &[Accessor]) -> Result<Arc<Value>, RuntimeError> {
		let reference = Reference {
			ref_type: ReferenceType::Local,
			namespace: Some(Arc::new(namespace.to_string())),
			variable: Arc::new(variable.to_string()),
			accessors: SmallVec::from(accessors.to_vec()),
		};
		self.resolve_reference(&reference)
	}

	pub fn add_value(&mut self, namespace: &str, key: &str, value: Value) -> Result<(), RuntimeError> {
		let namespace = Arc::new(namespace.to_string());
		let key = Arc::new(key.to_string());

		self.namespaces
			.entry(namespace)
			.or_insert_with(FnvHashMap::default)
			.insert(key, Arc::new(value));

		Ok(())
	}

	pub fn update_value(&mut self, namespace: &str, key: &str, value: Value) -> Result<(), RuntimeError> {
		let namespace = Arc::new(namespace.to_string());
		let key = Arc::new(key.to_string());

		match self.namespaces.get_mut(&namespace) {
			Some(ns) => {
				if ns.contains_key(&key) {
					ns.insert(key, Arc::new(value));
					Ok(())
				} else {
					Err(RuntimeError::VariableNotFound(key.to_string()))
				}
			}
			None => Err(RuntimeError::NamespaceNotFound(namespace.to_string()))
		}
	}

	pub fn delete_value(&mut self, namespace: &str, key: &str) -> Result<(), RuntimeError> {
		let namespace = Arc::new(namespace.to_string());
		let key = Arc::new(key.to_string());

		match self.namespaces.get_mut(&namespace) {
			Some(ns) => {
				if ns.remove(&key).is_some() {
					Ok(())
				} else {
					Err(RuntimeError::VariableNotFound(key.to_string()))
				}
			}
			None => Err(RuntimeError::NamespaceNotFound(namespace.to_string()))
		}
	}

	pub fn add_namespace(&mut self, namespace: &str) -> Result<(), RuntimeError> {
		let namespace = Arc::new(namespace.to_string());

		if !self.namespaces.contains_key(&namespace) {
			self.namespaces.insert(namespace, FnvHashMap::default());
			Ok(())
		} else {
			Err(RuntimeError::NamespaceAlreadyExists(namespace.to_string()))
		}
	}

	pub fn delete_namespace(&mut self, namespace: &str) -> Result<(), RuntimeError> {
		let namespace = Arc::new(namespace.to_string());

		if self.namespaces.remove(&namespace).is_some() {
			Ok(())
		} else {
			Err(RuntimeError::NamespaceNotFound(namespace.to_string()))
		}
	}

	pub fn list_namespaces(&self) -> Vec<&Arc<String>> {
		self.namespaces.keys().collect()
	}

	pub fn list_variables(&self, namespace: &str) -> Result<Vec<&Arc<String>>, RuntimeError> {
		self.namespaces
			.get(&Arc::new(namespace.to_string()))
			.map(|vars| vars.keys().collect())
			.ok_or_else(|| RuntimeError::NamespaceNotFound(namespace.to_string()))
	}

	fn resolve_reference(&self, reference: &Reference) -> Result<Arc<Value>, RuntimeError> {
		let mut visited = FnvHashSet::default();
		self.resolve_reference_recursive(reference, &mut visited, reference.namespace.clone())
	}

	fn resolve_reference_recursive(
		&self,
		reference: &Reference,
		visited: &mut FnvHashSet<(Arc<String>, Arc<String>)>,
		current_namespace: Option<Arc<String>>,
	) -> Result<Arc<Value>, RuntimeError> {
		let namespace = reference.namespace.as_ref().or(current_namespace.as_ref())
			.ok_or_else(|| RuntimeError::MissingNamespace)?;

		let key = (Arc::clone(namespace), Arc::clone(&reference.variable));
		if !visited.insert(key.clone()) {
			return Err(RuntimeError::CircularReference);
		}

		let variables = self.namespaces.get(namespace)
			.ok_or_else(|| RuntimeError::NamespaceNotFound(namespace.to_string()))?;
		let mut value = variables.get(&reference.variable)
			.ok_or_else(|| RuntimeError::VariableNotFound(reference.variable.to_string()))?
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
		value: Arc<Value>,
		visited: &mut FnvHashSet<(Arc<String>, Arc<String>)>,
	) -> Result<Arc<Value>, RuntimeError> {
		match &*value {
			Value::Intrinsic(_) => Err(RuntimeError::InvalidIntrinsicArgs),
			Value::Reference(ref inner_reference) => {
				let resolved_reference = Reference {
					ref_type: inner_reference.ref_type,
					namespace: inner_reference.namespace.clone().or_else(|| {
						visited.iter().next().map(|(ns, _)| ns.clone())
					}),
					variable: inner_reference.variable.clone(),
					accessors: inner_reference.accessors.clone(),
				};
				self.resolve_reference_recursive(&resolved_reference, visited, None)
			}
			Value::List(items) => {
				if let Some(Value::Intrinsic(_)) = items.first() {
					self.resolve_intrinsics(value, visited)
				} else {
					let resolved_items = items
						.iter()
						.map(|item| self.resolve_value(Arc::new(item.clone()), visited))
						// .map(|arc_val| (*arc_val).clone())
						.collect::<Result<Vec<_>, _>>()?;
					Ok(Arc::new(Value::List(Arc::new(
						resolved_items.into_iter().map(|arc| (*arc).clone()).collect(),
					))))
				}
			}
			_ => Ok(value),
		}
	}

	fn apply_accessor(&self, value: Arc<Value>, accessor: &Accessor) -> Result<Arc<Value>, RuntimeError> {
		match (&*value, accessor) {
			(Value::List(list), Accessor::Index(index)) => {
				list.get(*index)
					.map(|v| Arc::new(v.clone()))
					.ok_or(RuntimeError::IndexOutOfBounds(*index))
			}
			(Value::List(list), Accessor::Range(start, end)) => {
				if *start > *end || *end > list.len() {
					Err(RuntimeError::InvalidRange(*start, *end))
				} else {
					Ok(Arc::new(Value::List(Arc::new(list[*start..*end].to_vec()))))
				}
			}
			(Value::String(s), Accessor::Index(index)) => {
				s.chars()
					.nth(*index)
					.map(|c| Arc::new(Value::String(c.to_string())))
					.ok_or(RuntimeError::IndexOutOfBounds(*index))
			}
			(Value::String(s), Accessor::Range(start, end)) => {
				if *start > *end || *end > s.len() {
					Err(RuntimeError::InvalidRange(*start, *end))
				} else {
					Ok(Arc::new(Value::String(s[*start..*end].to_string())))
				}
			}
			(_, Accessor::Key(key)) => {
				Err(RuntimeError::InvalidAccessor(format!(
					"Key accessor '{}' not supported for this value type",
					key
				)))
			}
			_ => Err(RuntimeError::InvalidAccessor(
				"Accessor not supported for this value type".to_string(),
			)),
		}
	}

	pub fn resolve_intrinsics(
		&self,
		value: Arc<Value>,
		visited: &mut FnvHashSet<(Arc<String>, Arc<String>)>,
	) -> Result<Arc<Value>, RuntimeError> {
		match &*value {
			Value::Intrinsic(_) => Err(RuntimeError::InvalidIntrinsicArgs),
			Value::List(items) => {
				if let Some(Value::Intrinsic(name)) = items.first() {
					if let Some(func) = self.std_lib_loader.get_function(name) {
						let args = self.validate_and_collect_args(name, items, visited)?;
						Ok(func(args))
					} else {
						Err(RuntimeError::UnknownIntrinsic(name.clone()))
					}
				} else {
					let resolved_items = items
						.iter()
						.map(|item| {
							self.resolve_intrinsics(Arc::new(item.clone()), visited)
								.map(|arc_val| (*arc_val).clone())
						})
						.collect::<Result<Vec<Value>, _>>()?;
					Ok(Arc::new(Value::List(Arc::new(resolved_items))))
				}
			}
			_ => Ok(value),
		}
	}

	fn validate_and_collect_args(
		&self,
		name: &str,
		items: &[Value],
		visited: &mut FnvHashSet<(Arc<String>, Arc<String>)>
	) -> Result<Vec<Arc<Value>>, RuntimeError> {
		let args = items.iter().skip(1); // Skip intrinsic name
		let arg_count = args.clone().count();

		if arg_count == 0 {
			return Err(RuntimeError::InvalidIntrinsicArgs);
		}

		if name.starts_with("std") {
			let expected_count = match name {
				// Single argument functions
				"std_to_uppercase" | "std_to_lowercase" | "std_base64_encode" |
				"std_base64_decode" | "std_float_to_int" | "std_int_to_float" |
				"std_bitwise_not" => 1,

				// Two argument functions
				"std_add_int" | "std_sub_int" | "std_mul_int" | "std_div_int" |
				"std_mod_int" | "std_add_float" | "std_sub_float" | "std_mul_float" |
				"std_div_float" | "std_try" | "std_lt" | "std_gt" | "std_eq" |
				"std_bitwise_and" | "std_bitwise_or" | "std_bitwise_xor" => 2,

				// Three argument functions
				"std_substring" | "std_if" | "std_replace" | "std_concat" => 3,

				// Special cases
				"std_hash" => 2,

				_ => return Err(RuntimeError::UnknownIntrinsic(name.to_string())),
			};

			if arg_count != expected_count {
				return Err(RuntimeError::InvalidIntrinsicArgs);
			}
		}

		let resolved_args = items.iter()
			.skip(1)
			.map(|item| self.resolve_value(Arc::new(item.clone()), visited))
			.collect::<Result<Vec<_>, _>>()?;

		// Validate argument types
		for (idx, arg) in resolved_args.iter().enumerate() {
			match (name, idx) {
				// Integer operations
				("std_add_int" | "std_sub_int" | "std_mul_int" | "std_div_int" |
				"std_mod_int" | "std_bitwise_and" | "std_bitwise_or" |
				"std_bitwise_xor", _) => {
					if !matches!(&**arg, Value::Number(Number::Integer(_))) {
						return Err(RuntimeError::IntrinsicTypeMismatch(
							format!("{} requires integer arguments", name)
						));
					}
				}

				// Float operations
				("std_add_float" | "std_sub_float" | "std_mul_float" |
				"std_div_float", _) => {
					if !matches!(&**arg, Value::Number(Number::Float(_))) {
						return Err(RuntimeError::IntrinsicTypeMismatch(
							format!("{} requires float arguments", name)
						));
					}
				}

				// String operations
				("std_to_uppercase" | "std_to_lowercase" | "std_base64_encode" |
				"std_base64_decode", 0) => {
					if !matches!(&**arg, Value::String(_)) {
						return Err(RuntimeError::IntrinsicTypeMismatch(
							format!("{} requires string argument", name)
						));
					}
				}

				// Substring operation
				("std_substring", 0) | ("std_replace", 0) => {
					if !matches!(&**arg, Value::String(_)) {
						return Err(RuntimeError::IntrinsicTypeMismatch(
							format!("{} first argument must be string", name)
						));
					}
				}
				("std_substring", 1..=2) => {
					if !matches!(&**arg, Value::Number(Number::Integer(_))) {
						return Err(RuntimeError::IntrinsicTypeMismatch(
							"std_substring indices must be integers".to_string()
						));
					}
				}

				// Replace operation
				("std_replace", 1..=2) => {
					if !matches!(&**arg, Value::String(_)) {
						return Err(RuntimeError::IntrinsicTypeMismatch(
							"std_replace requires string arguments".to_string()
						));
					}
				}

				// Conversion operations
				("std_int_to_float", 0) => {
					if !matches!(&**arg, Value::Number(Number::Integer(_))) {
						return Err(RuntimeError::IntrinsicTypeMismatch(
							"std_int_to_float requires integer argument".to_string()
						));
					}
				}
				("std_float_to_int", 0) => {
					if !matches!(&**arg, Value::Number(Number::Float(_))) {
						return Err(RuntimeError::IntrinsicTypeMismatch(
							"std_float_to_int requires float argument".to_string()
						));
					}
				}

				// Control flow
				("std_if", 0) => {
					if !matches!(&**arg, Value::Boolean(_)) {
						return Err(RuntimeError::IntrinsicTypeMismatch(
							"std_if first argument must be boolean".to_string()
						));
					}
				}

				// Hash operation
				("std_hash", 0) => {
					if !matches!(&**arg, Value::String(_)) {
						return Err(RuntimeError::IntrinsicTypeMismatch(
							"std_hash first argument must be string".to_string()
						));
					}
				}
				("std_hash", 1) => {
					if !matches!(&**arg, Value::String(_)) {
						return Err(RuntimeError::IntrinsicTypeMismatch(
							"std_hash second argument must be string".to_string()
						));
					}
				}

				_ => {}
			}
		}

		Ok(resolved_args)
	}

	fn collect_intrinsic_args(
		&self,
		items: &[Value],
		visited: &mut FnvHashSet<(Arc<String>, Arc<String>)>,
	) -> Result<Vec<Arc<Value>>, RuntimeError> {
		items.iter()
			.skip(1)
			.map(|item| self.resolve_value(Arc::new(item.clone()), visited))
			.collect()
	}
}