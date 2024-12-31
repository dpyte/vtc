use std::fmt;

use crate::value::{
	Accessor, Namespace, Number, Reference, ReferenceType, Value, Variable, VtcFile,
};

impl fmt::Display for VtcFile {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		for (i, namespace) in self.namespaces.iter().enumerate() {
			write!(f, "{}", namespace)?;
			if i < self.namespaces.len() - 1 {
				writeln!(f)?;
			}
		}
		Ok(())
	}
}

impl fmt::Display for Namespace {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		writeln!(f, "@{}:", self.name)?;
		for (i, variable) in self.variables.iter().enumerate() {
			write!(f, "    {}", variable)?;
			if i < self.variables.len() - 1 {
				writeln!(f)?;
			}
		}
		Ok(())
	}
}

impl fmt::Display for Variable {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		write!(f, "${} := {}", self.name, self.value)
	}
}

impl fmt::Display for Value {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		match self {
			Value::Intrinsic(i) => write!(f, "\"{}\"", i),
			Value::String(s) => write!(f, "\"{}\"", s),
			Value::Number(n) => write!(f, "{}", n),
			Value::Boolean(b) => write!(f, "{}", b),
			Value::Nil => write!(f, "Nil"),
			Value::List(l) => {
				write!(f, "[")?;
				for (i, item) in l.iter().enumerate() {
					if i > 0 {
						write!(f, ", ")?;
					}
					write!(f, "{}", item)?;
				}
				write!(f, "]")
			}
			Value::Reference(r) => write!(f, "{}", r),
		}
	}
}

impl fmt::Display for Number {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		match self {
			Number::Integer(i) => write!(f, "{}", i),
			Number::Float(fl) => write!(f, "{}", fl),
			Number::Binary(b) => write!(f, "0b{:b}", b),
			Number::Hexadecimal(h) => write!(f, "0x{:X}", h),
		}
	}
}

impl fmt::Display for Reference {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		match self.ref_type {
			ReferenceType::External => write!(f, "&")?,
			ReferenceType::Local => write!(f, "%")?,
		}
		if let Some(ns) = &self.namespace {
			write!(f, "{}.", ns)?;
		}
		write!(f, "{}", self.variable)?;
		for accessor in &self.accessors {
			write!(f, "->{}", accessor)?;
		}
		Ok(())
	}
}

impl fmt::Display for Accessor {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		match self {
			Accessor::Index(i) => write!(f, "({})", i),
			Accessor::Range(start, end) => write!(f, "({}..{})", start, end),
			Accessor::Key(k) => write!(f, "[{}]", k),
		}
	}
}
