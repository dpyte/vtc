use std::rc::Rc;

pub mod parser;
pub mod runtime;
pub mod ffi;

pub const SMALL_VEC_SIZE: usize = 64;

use smallvec::SmallVec;

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
	Intrinsic(Rc<String>),
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