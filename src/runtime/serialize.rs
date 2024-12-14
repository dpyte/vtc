use std::collections::HashSet;
use std::fmt::Write as FmtWrite;
use std::fs::{File, OpenOptions};
use std::io::{self, BufWriter, Write};
use std::path::Path;
use std::sync::Arc;

use fnv::FnvHashMap;

use crate::runtime::Runtime;
use crate::value::{Accessor, Number, Reference, ReferenceType, Value};

const INITIAL_BUFFER_SIZE: usize = 4096;

impl Runtime {
	pub fn dump_to_file<P: AsRef<Path>>(&self, path: P) -> io::Result<()> {
		if let Some(parent) = path.as_ref().parent() {
			std::fs::create_dir_all(parent)?;
		}

		let file = File::create(path)?;
		let mut writer = BufWriter::with_capacity(INITIAL_BUFFER_SIZE, file);

		for (namespace, variables) in &self.namespaces {
			self.write_namespace(&mut writer, namespace, variables)?;
		}

		writer.flush()?;
		Ok(())
	}

	pub fn dump_selective<P: AsRef<Path>>(&self, path: P, namespaces: Vec<String>) -> io::Result<()> {
		let file = OpenOptions::new()
			.write(true)
			.truncate(true)
			.create(true)
			.open(path)?;
		let mut writer = BufWriter::with_capacity(INITIAL_BUFFER_SIZE, file);

		let mut dumped_namespaces = HashSet::with_capacity(self.namespaces.len());
		let mut to_dump = if namespaces.is_empty() {
			self.namespaces.keys().cloned().collect::<Vec<_>>()
		} else {
			namespaces.into_iter().map(Arc::from).collect()
		};

		while let Some(namespace) = to_dump.pop() {
			if dumped_namespaces.contains(&namespace) {
				continue;
			}
			if let Some(variables) = self.namespaces.get(&namespace) {
				self.write_namespace_selective(
					&mut writer,
					&namespace,
					variables,
					&mut dumped_namespaces,
					&mut to_dump,
				)?;
			}
		}

		writer.flush()?;
		Ok(())
	}

	fn write_namespace(
		&self,
		writer: &mut BufWriter<File>,
		namespace: &Arc<String>,
		variables: &FnvHashMap<Arc<String>, Arc<Value>>
	) -> io::Result<()> {
		writeln!(writer, "@{}:", namespace)?;

		for (var_name, value) in variables {
			writeln!(writer, "\t${} := {}", var_name, self.serialize_value(value))?;
		}

		writeln!(writer)?;
		Ok(())
	}

	fn write_namespace_selective(
		&self,
		writer: &mut BufWriter<File>,
		namespace: &Arc<String>,
		variables: &FnvHashMap<Arc<String>, Arc<Value>>,
		dumped_namespaces: &mut HashSet<Arc<String>>,
		to_dump: &mut Vec<Arc<String>>,
	) -> io::Result<()> {
		writeln!(writer, "@{}:", namespace)?;

		let mut vars: Vec<_> = variables.iter().collect();
		vars.sort_by(|(k1, _), (k2, _)| k1.cmp(k2));

		for (var_name, value) in vars {
			let serialized = self.serialize_value_selective(value, dumped_namespaces, to_dump);
			writeln!(writer, "\t${} := {}", var_name, serialized)?;
		}

		writeln!(writer)?;
		dumped_namespaces.insert(Arc::clone(namespace));
		Ok(())
	}

	pub fn serialize_value(&self, value: &Value) -> String {
		let mut buffer = String::with_capacity(64);
		self.serialize_value_to_string(value, &mut buffer);
		buffer
	}

	fn serialize_value_to_string(&self, value: &Value, buffer: &mut String) {
		match value {
			Value::String(s) => {
				if s.contains('"') {
					buffer.push_str(s);
				} else {
					buffer.push('"');
					buffer.push_str(s);
					buffer.push('"');
				}
			},
			Value::Number(num) => match num {
				Number::Integer(i) => buffer.push_str(&i.to_string()),
				Number::Float(f) => buffer.push_str(&f.to_string()),
				Number::Binary(b) => write!(buffer, "0b{:b}", b).unwrap(),
				Number::Hexadecimal(h) => write!(buffer, "0x{:X}", h).unwrap(),
			},
			Value::Boolean(b) => buffer.push_str(if *b { "True" } else { "False" }),
			Value::List(list) => {
				buffer.push('[');
				for (i, item) in list.iter().enumerate() {
					if i > 0 {
						buffer.push_str(", ");
					}
					self.serialize_value_to_string(item, buffer);
				}
				buffer.push(']');
			},
			Value::Reference(ref_val) => {
				self.serialize_reference_to_string(ref_val, buffer);
			},
			Value::Intrinsic(name) => {
				buffer.push_str(name);
				buffer.push_str("!!");
			},
			Value::Nil => buffer.push_str("Nil"),
		}
	}

	fn serialize_value_selective(
		&self,
		value: &Value,
		dumped_namespaces: &mut HashSet<Arc<String>>,
		to_dump: &mut Vec<Arc<String>>,
	) -> String {
		let mut buffer = String::with_capacity(64);
		match value {
			Value::List(list) => {
				buffer.push('[');
				for (i, item) in list.iter().enumerate() {
					if i > 0 {
						buffer.push_str(", ");
					}
					buffer.push_str(&self.serialize_value_selective(item, dumped_namespaces, to_dump));
				}
				buffer.push(']');
			},
			Value::Reference(ref_val) => {
				self.serialize_reference_selective(ref_val, dumped_namespaces, to_dump, &mut buffer);
			},
			_ => self.serialize_value_to_string(value, &mut buffer),
		}
		buffer
	}

	fn serialize_reference_to_string(&self, ref_val: &Reference, buffer: &mut String) {
		match ref_val.ref_type {
			ReferenceType::External => buffer.push('&'),
			ReferenceType::Local => buffer.push('%'),
		}

		if let Some(ns) = &ref_val.namespace {
			buffer.push_str(ns);
			buffer.push('.');
		}
		buffer.push_str(&ref_val.variable);

		for acc in &ref_val.accessors {
			match acc {
				Accessor::Index(i) => write!(buffer, "->({})", i).unwrap(),
				Accessor::Range(start, end) => write!(buffer, "->({}, {})", start, end).unwrap(),
				Accessor::Key(key) => write!(buffer, "->[{}]", key).unwrap(),
			}
		}
	}

	fn serialize_reference_selective(
		&self,
		ref_val: &Reference,
		dumped_namespaces: &mut HashSet<Arc<String>>,
		to_dump: &mut Vec<Arc<String>>,
		buffer: &mut String,
	) {
		if let Some(ns) = &ref_val.namespace {
			if !dumped_namespaces.contains(ns) {
				to_dump.push(Arc::from(ns.to_string()));
			}
		}
		self.serialize_reference_to_string(ref_val, buffer);
	}
}