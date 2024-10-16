use crate::value::{Number, Value};
use std::fs::File;
use std::io;
use std::io::Write;
use std::path::Path;
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
}
