use std::collections::{HashMap, HashSet};
use crate::value::{Number, Reference, Value};
use std::fs::{File, OpenOptions};
use std::io;
use std::io::Write;
use std::path::Path;
use std::rc::Rc;
use crate::runtime::Runtime;
use crate::value::{Accessor};

impl Runtime {
	/// Dumps the entire runtime state to a file in VTC format.
	///
	/// This function writes all namespaces and their variables to the specified file,
	/// maintaining the VTC syntax structure.
	///
	/// # Arguments
	///
	/// * `path` - A path-like object representing the file to write to.
	///
	/// # Returns
	///
	/// A `Result` which is:
	/// - `Ok(())` if the write operation was successful.
	/// - `Err(e)` if there was an error writing to the file, where `e` is the I/O error.
	///
	/// # Examples
	///
	/// ```
	/// use vtc::runtime::Runtime;
	///
	/// let runtime = Runtime::new();
	/// // ... populate the runtime ...
	/// runtime.dump_to_file("output.vtc").expect("Failed to dump runtime");
	/// ```
	pub fn dump_to_file<P: AsRef<Path>>(&self, path: P) -> io::Result<()> {
		let mut file = File::create(path)?;

		for (namespace, variables) in &self.namespaces {
			file.write_all(format!("@{}:\n", namespace).as_bytes())?;
			for (var_name, value) in variables {
				file.write_all(format!("    ${} := {}\n", var_name, self.serialize_value(value)).as_bytes())?;
			}
			file.write_all(b"\n")?;  // Add a blank line between namespaces
		}

		Ok(())
	}

	/// Serializes a `Value` into its string representation in VTC syntax.
	///
	/// This function converts different types of `Value` into their corresponding
	/// string representation according to the VTC syntax rules.
	///
	/// # Arguments
	///
	/// * `value` - A reference to the `Value` to be serialized.
	///
	/// # Returns
	///
	/// A `String` representing the serialized form of the `Value`.
	///
	/// # Examples
	///
	/// ```
	/// use vtc::value::{Number, Value};
	/// let value = Value::Number(Number::Integer(42));
	/// let serialized = runtime.serialize_value(&value);
	/// assert_eq!(serialized, "42");
	/// ```
	fn serialize_value(&self, value: &Value) -> String {
		match value {
			Value::String(s) => format!("\"{}\"", s),
			Value::Number(Number::Integer(i)) => i.to_string(),
			Value::Number(Number::Float(f)) => f.to_string(),
			Value::Boolean(b) => b.to_string(),
			Value::List(list) => {
				let items: Vec<String> = list.iter()
					.map(|item| self.serialize_value(item))
					.collect();
				format!("[{}]", items.join(", "))
			},
			Value::Reference(ref_val) => {
				let namespace = ref_val.namespace.as_ref()
					.map(|ns| format!("{}.", ns))
					.unwrap_or_default();
				let accessors = ref_val.accessors.iter()
					.map(|acc| match acc {
						Accessor::Index(i) => format!("->({})", i),
						Accessor::Range(start, end) => format!("->({}, {})", start, end),
						Accessor::Key(key) => format!("->[{}]", key),
					})
					.collect::<String>();
				format!("%{}{}{}", namespace, ref_val.variable, accessors)
			},
			Value::Intrinsic(name) => format!("{}!!", name),
			Value::Nil => "Nil".to_string(),
			Value::Number(Number::Binary(b)) => format!("0b{:064b}", b),
			Value::Number(Number::Hexadecimal(hx)) => format!("0x{:016X}", hx),
		}
	}

	/// Serializes a `Value` for selective dumping, tracking dependencies.
	fn serialize_value_selective(
		&self,
		value: &Value,
		dumped_namespaces: &mut HashSet<Rc<String>>,
		to_dump: &mut Vec<Rc<String>>,
	) -> String {
		match value {
			Value::List(list) => {
				let items: Vec<String> = list.iter()
					.map(|item| self.serialize_value_selective(item, dumped_namespaces, to_dump))
					.collect();
				format!("[{}]", items.join(", "))
			},
			Value::Reference(ref_val) => self.serialize_reference_selective(ref_val, dumped_namespaces, to_dump),
			_ => self.serialize_value(value)
		}
	}

	fn serialize_reference_selective(
		&self,
		ref_val: &Reference,
		dumped_namespaces: &mut HashSet<Rc<String>>,
		to_dump: &mut Vec<Rc<String>>,
	) -> String {
		if let Some(ns) = &ref_val.namespace {
			if !dumped_namespaces.contains(ns) {
				to_dump.push(Rc::clone(ns));
			}
		}
		self.serialize_value(&Value::Reference(ref_val.clone().into()))
	}

	pub fn dump_selective<P: AsRef<Path>>(&self, path: P, namespaces: Vec<String>) -> io::Result<()> {
		let mut file = OpenOptions::new()
			.write(true)
			.truncate(true)
			.create(true)
			.open(path)?;

		let mut dumped_namespaces = HashSet::new();
		let mut to_dump = if namespaces.is_empty() {
			self.namespaces.keys().cloned().collect::<Vec<_>>()
		} else {
			namespaces.into_iter().map(Rc::from).collect()
		};

		while let Some(namespace) = to_dump.pop() {
			if dumped_namespaces.contains(&namespace) {
				continue;
			}
			if let Some(variables) = self.namespaces.get(&namespace) {
				self.dump_namespace(&mut file, &namespace, variables, &mut dumped_namespaces, &mut to_dump)?;
			}
		}
		Ok(())
	}

	fn dump_namespace(
		&self,
		file: &mut File,
		namespace: &Rc<String>,
		variables: &HashMap<Rc<String>, Rc<Value>>,
		dumped_namespaces: &mut HashSet<Rc<String>>,
		to_dump: &mut Vec<Rc<String>>,
	) -> io::Result<()> {
		writeln!(file, "@{}:", namespace)?;

		for (var_name, value) in variables {
			let serialized = self.serialize_value_selective(value, dumped_namespaces, to_dump);
			writeln!(file, "    ${} := {}", var_name, serialized)?;
		}

		writeln!(file)?;  // Add a blank line between namespaces
		dumped_namespaces.insert(Rc::clone(namespace));

		Ok(())
	}

	fn dump_namespace_selective(
		&self,
		file: &mut File,
		namespace: &Rc<String>,
		variables: &HashMap<Rc<String>, Rc<Value>>,
		dumped_namespaces: &mut HashSet<Rc<String>>,
		to_dump: &mut Vec<Rc<String>>,
	) -> io::Result<()> {
		file.write_all(format!("@{}:\n", namespace).as_bytes())?;

		for (var_name, value) in variables {
			let serialized = self.serialize_value_selective(value, dumped_namespaces, to_dump);
			file.write_all(format!("    ${} := {}\n", var_name, serialized).as_bytes())?;
		}

		file.write_all(b"\n")?;  // Add a blank line between namespaces
		dumped_namespaces.insert(Rc::clone(namespace));

		Ok(())
	}
}
