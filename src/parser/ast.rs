use std::fmt;
use std::rc::Rc;

use smallvec::SmallVec;

use crate::SMALL_VEC_SIZE;

#[derive(Debug, Clone, PartialEq)]
pub struct VtcFile {
	pub namespaces: Vec<Rc<Namespace>>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Namespace {
	pub name: Rc<String>,
	pub variables: Vec<Rc<Variable>>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Variable {
	pub name: Rc<String>,
	pub value: Rc<Value>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Value {
	String(Rc<String>),
	Number(Number),
	Boolean(bool),
	Nil,
	List(Rc<Vec<Rc<Value>>>),
	Reference(Rc<Reference>),
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Number {
	Integer(i64),
	Float(f64),
	Binary(i64),
	Hexadecimal(i64),
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ReferenceType {
	External, // &
	Local,    // %
}

#[derive(Debug, Clone, PartialEq)]
pub struct Reference {
	pub ref_type: ReferenceType,
	pub namespace: Option<Rc<String>>,
	pub variable: Rc<String>,
	pub accessors: SmallVec<[Accessor; SMALL_VEC_SIZE]>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Accessor {
	Index(usize),
	Range(usize, usize),
	Key(Rc<String>),
}

// Implement Display traits (unchanged from previous version)
impl fmt::Display for VtcFile {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		for namespace in &self.namespaces {
			write!(f, "{}", namespace)?;
			if !Rc::ptr_eq(namespace, self.namespaces.last().unwrap()) {
				writeln!(f)?;
			}
		}
		Ok(())
	}
}

impl fmt::Display for Namespace {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		writeln!(f, "@{}:", self.name)?;
		for variable in &self.variables {
			write!(f, "    {}", variable)?;
			if !Rc::ptr_eq(variable, self.variables.last().unwrap()) {
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