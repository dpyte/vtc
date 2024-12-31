//! Runtime module for the VTC (Variable Type Configuration) system.
//!
//! This module provides the core runtime environment for executing and managing VTC programs.
//! It handles loading, parsing, and executing VTC files while managing namespaces, variables,
//! and intrinsic functions.
//!
//! # Key Features
//!
//! - Thread-safe runtime environment
//! - Namespace-based variable management
//! - Support for intrinsic functions
//! - Reference resolution with cycle detection
//! - Value accessors for lists and strings
//! - Comprehensive error handling
//!
//! # Example
//!
//! ```
//!
//! // Create a new runtime
//! use vtc::runtime::Runtime;
//!
//! let mut runtime = Runtime::new();
//!
//! // Load a VTC file
//! runtime.load_vtc(r#"
//!     @namespace:
//!         $variable := ["Hello, World!"]
//! "#).unwrap();
//!
//! // Access a value
//! let value = runtime.get_value("namespace", "variable", &[]).unwrap();
//! ```

use ::std::{fmt, fs};
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


/// A thread-safe runtime environment for VTC program execution.
///
/// The Runtime struct is the core component that manages the execution environment
/// for VTC programs. It maintains a collection of namespaces and their associated
/// variables, handles reference resolution, and provides access to standard library
/// functions.
///
/// # Thread Safety
///
/// The runtime uses Arc (Atomic Reference Counting) for thread-safe sharing of values
/// and FnvHashMap for efficient hash-based storage.
///
/// # Fields
///
/// * `namespaces` - A thread-safe map of namespaces to their variables
/// * `std_lib_loader` - Loader for standard library functions
pub struct Runtime {
	pub namespaces: FnvHashMap<Arc<String>, FnvHashMap<Arc<String>, Arc<Value>>>,
	std_lib_loader: StdLibLoader
}

impl fmt::Debug for Runtime {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		f.debug_struct("Runtime")
			.field("namespaces", &self.namespaces)
			.finish()
	}
}

impl Runtime {
	/// Creates a new empty runtime environment.
	///
	/// # Returns
	///
	/// A new Runtime instance with initialized namespaces and standard library.
	pub fn new() -> Self {
		Runtime {
			namespaces: FnvHashMap::default(),
			std_lib_loader: StdLibLoader::new(),
		}
	}

	/// Creates a runtime environment from a VTC file at the specified path.
	///
	/// # Arguments
	///
	/// * `path` - Path to the VTC file
	///
	/// # Returns
	///
	/// A Result containing either the new Runtime or a RuntimeError
	///
	/// # Errors
	///
	/// Returns RuntimeError if:
	/// * The file cannot be read
	/// * The file contains invalid VTC syntax
	/// * Any referenced namespaces or variables are missing
	pub fn from_file(path: PathBuf) -> Result<Self, RuntimeError> {
		let mut rt = Self::new();
		rt.load_file(path)?;
		Ok(rt)
	}

	/// Creates a runtime environment from a VTC string.
	///
	/// # Arguments
	///
	/// * `input` - String containing VTC code
	///
	/// # Returns
	///
	/// A Result containing either the new Runtime or a RuntimeError
	///
	/// # Errors
	///
	/// Returns RuntimeError if:
	/// * The input contains invalid VTC syntax
	/// * Any referenced namespaces or variables are missing
	pub fn from_str(input: &str) -> Result<Self, RuntimeError> {
		let mut rt = Runtime::new();
		rt.load_vtc(input)?;
		Ok(rt)
	}

	/// Creates a new Runtime instance from an existing VtcFile structure.
	///
	/// This method creates a fresh runtime environment and populates it with
	/// the namespaces and variables from the provided VtcFile. This is useful
	/// when you have already parsed a VTC configuration and want to create a
	/// new runtime instance from it.
	///
	/// # Arguments
	///
	/// * `vtc` - A VtcFile structure containing the parsed configuration
	///
	/// # Returns
	///
	/// * `Ok(Runtime)` - A new Runtime instance initialized with the VtcFile content
	/// * `Err(RuntimeError)` - If there was an error loading the VtcFile
	///
	/// # Errors
	///
	/// Returns RuntimeError if:
	/// * There are duplicate namespace definitions
	/// * Variable references cannot be resolved
	/// * Circular references are detected
	///
	/// # Example
	///
	/// ```
	///
	/// use vtc::runtime::Runtime;
	/// use vtc::value::{Namespace, VtcFile};
	///
	/// let vtc_file = VtcFile {
	///     namespaces: vec![
	///         Namespace {
	///             name: String::from("test"),
	///             variables: vec![]
	///         }
	///     ]
	/// };
	///
	/// let runtime = Runtime::copy_from_vtc_file(vtc_file).unwrap();
	/// ```
	pub fn copy_from_vtc_file(vtc: VtcFile) -> Result<Self, RuntimeError> {
		let mut rt = Runtime::new();
		rt.load_vtc_file(vtc)?;
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

	pub fn update_library_loader(&mut self, lib_loader: StdLibLoader) -> Result<(), RuntimeError> {
		self.std_lib_loader = lib_loader;
		Ok(())
	}

	/// Retrieves a value from the runtime.
	///
	/// # Arguments
	///
	/// * `namespace` - The namespace containing the variable
	/// * `variable` - The name of the variable to retrieve
	/// * `accessors` - Optional accessors for indexing into lists or strings
	///
	/// # Returns
	///
	/// A Result containing either the requested Value or a RuntimeError
	///
	/// # Errors
	///
	/// Returns RuntimeError if:
	/// * The namespace does not exist
	/// * The variable does not exist
	/// * The accessors are invalid for the value type
	/// * A circular reference is detected
	pub fn get_value(&self, namespace: &str, variable: &str, accessors: &[Accessor]) -> Result<Arc<Value>, RuntimeError> {
		let reference = Reference {
			ref_type: ReferenceType::Local,
			namespace: Some(Arc::new(namespace.to_string())),
			variable: Arc::new(variable.to_string()),
			accessors: SmallVec::from(accessors.to_vec()),
		};
		self.resolve_reference(&reference)
	}

	/// Adds a new value to the specified namespace.
	///
	/// If the namespace doesn't exist, it will be created automatically.
	///
	/// # Arguments
	///
	/// * `namespace` - The target namespace
	/// * `key` - The variable name
	/// * `value` - The value to store
	///
	/// # Returns
	///
	/// Ok(()) if successful, or RuntimeError if the operation fails
	pub fn add_value(&mut self, namespace: &str, key: &str, value: Value) -> Result<(), RuntimeError> {
		let namespace = Arc::new(namespace.to_string());
		let key = Arc::new(key.to_string());

		self.namespaces
			.entry(namespace)
			.or_insert_with(FnvHashMap::default)
			.insert(key, Arc::new(value));

		Ok(())
	}

	/// Updates an existing value in the specified namespace.
	///
	/// # Arguments
	///
	/// * `namespace` - The target namespace
	/// * `key` - The variable name to update
	/// * `value` - The new value
	///
	/// # Returns
	///
	/// Ok(()) if successful
	///
	/// # Errors
	///
	/// Returns RuntimeError if:
	/// * The namespace doesn't exist
	/// * The variable doesn't exist
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

	/// Removes a value from the specified namespace.
	///
	/// # Arguments
	///
	/// * `namespace` - The target namespace
	/// * `key` - The variable name to remove
	///
	/// # Returns
	///
	/// Ok(()) if successful
	///
	/// # Errors
	///
	/// Returns RuntimeError if:
	/// * The namespace doesn't exist
	/// * The variable doesn't exist
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

	/// Creates a new empty namespace.
	///
	/// # Arguments
	///
	/// * `namespace` - The name of the namespace to create
	///
	/// # Returns
	///
	/// Ok(()) if successful
	///
	/// # Errors
	///
	/// Returns RuntimeError::NamespaceAlreadyExists if the namespace already exists
	pub fn add_namespace(&mut self, namespace: &str) -> Result<(), RuntimeError> {
		let namespace = Arc::new(namespace.to_string());

		if !self.namespaces.contains_key(&namespace) {
			self.namespaces.insert(namespace, FnvHashMap::default());
			Ok(())
		} else {
			Err(RuntimeError::NamespaceAlreadyExists(namespace.to_string()))
		}
	}

	/// Removes an entire namespace and all its variables.
	///
	/// # Arguments
	///
	/// * `namespace` - The name of the namespace to remove
	///
	/// # Returns
	///
	/// Ok(()) if successful
	///
	/// # Errors
	///
	/// Returns RuntimeError::NamespaceNotFound if the namespace doesn't exist
	pub fn delete_namespace(&mut self, namespace: &str) -> Result<(), RuntimeError> {
		let namespace = Arc::new(namespace.to_string());

		if self.namespaces.remove(&namespace).is_some() {
			Ok(())
		} else {
			Err(RuntimeError::NamespaceNotFound(namespace.to_string()))
		}
	}


	/// Returns a list of all namespace names in the runtime.
	///
	/// # Returns
	///
	/// A vector of references to namespace names
	pub fn list_namespaces(&self) -> Vec<&Arc<String>> {
		self.namespaces.keys().collect()
	}


	/// Lists all variables in a specified namespace.
	///
	/// # Arguments
	///
	/// * `namespace` - The namespace to list variables from
	///
	/// # Returns
	///
	/// A Result containing either a vector of variable names or RuntimeError
	///
	/// # Errors
	///
	/// Returns RuntimeError::NamespaceNotFound if the namespace doesn't exist
	pub fn list_variables(&self, namespace: &str) -> Result<Vec<&Arc<String>>, RuntimeError> {
		self.namespaces
			.get(&Arc::new(namespace.to_string()))
			.map(|vars| vars.keys().collect())
			.ok_or_else(|| RuntimeError::NamespaceNotFound(namespace.to_string()))
	}

	/// Resolves a reference to its actual value, handling variable lookups and accessors.
	///
	/// # Arguments
	///
	/// * `reference` - The reference to resolve
	///
	/// # Returns
	///
	/// The resolved value or RuntimeError if resolution fails
	///
	/// # Errors
	///
	/// Returns RuntimeError if:
	/// * Circular references are detected
	/// * Referenced namespace or variable doesn't exist
	/// * Invalid accessors are encountered
	fn resolve_reference(&self, reference: &Reference) -> Result<Arc<Value>, RuntimeError> {
		let mut visited = FnvHashSet::default();
		self.resolve_reference_recursive(reference, &mut visited, reference.namespace.clone())
	}

	/// Recursively resolves a reference while tracking visited references to prevent cycles.
	///
	/// # Arguments
	///
	/// * `reference` - The reference to resolve
	/// * `visited` - Set of already visited references
	/// * `current_namespace` - The current namespace context
	///
	/// # Returns
	///
	/// The resolved value or RuntimeError if resolution fails
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

	/// Applies an accessor to a value (e.g., list indexing or string slicing).
	///
	/// # Arguments
	///
	/// * `value` - The value to access
	/// * `accessor` - The accessor to apply
	///
	/// # Returns
	///
	/// The accessed value or RuntimeError if the accessor is invalid
	///
	/// # Errors
	///
	/// Returns RuntimeError if:
	/// * Index is out of bounds
	/// * Range is invalid
	/// * Accessor type is not supported for the value type
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

	/// Resolves and validates intrinsic function calls.
	///
	/// # Arguments
	///
	/// * `value` - The value containing the intrinsic function call
	/// * `visited` - Set of visited references during resolution
	///
	/// # Returns
	///
	/// The result of the intrinsic function call or RuntimeError if validation fails
	pub fn resolve_intrinsics(
		&self,
		value: Arc<Value>,
		visited: &mut FnvHashSet<(Arc<String>, Arc<String>)>,
	) -> Result<Arc<Value>, RuntimeError> {
		match &*value {
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
			},
			_ => Ok(value)
		}
	}

	/// Validates and collects arguments for intrinsic function calls.
	///
	/// Performs type checking and argument count validation for standard library functions.
	///
	/// # Arguments
	///
	/// * `name` - Name of the intrinsic function
	/// * `items` - List of argument values
	/// * `visited` - Set of visited references during resolution
	///
	/// # Returns
	///
	/// Vector of resolved argument values or RuntimeError if validation fails
	fn validate_and_collect_args(
		&self,
		name: &str,
		items: &[Value],
		visited: &mut FnvHashSet<(Arc<String>, Arc<String>)>
	) -> Result<Vec<Arc<Value>>, RuntimeError> {
		let args = items.iter().skip(1);
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