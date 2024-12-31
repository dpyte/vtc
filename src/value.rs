use std::sync::Arc;

use smallvec::SmallVec;

use crate::SMALL_VEC_SIZE;

#[derive(Debug, Clone, PartialEq)]
pub struct VtcFile {
    pub namespaces: Vec<Namespace>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Namespace {
    pub name: String,
    pub variables: Vec<Variable>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Variable {
    pub name: String,
    pub value: Value,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Value {
    Intrinsic(String),
    String(String),
    Number(Number),
    Boolean(bool),
    Nil,
    List(Arc<Vec<Value>>),
    Reference(Reference),
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
    pub namespace: Option<Arc<String>>,
    pub variable: Arc<String>,
    pub accessors: SmallVec<[Accessor; SMALL_VEC_SIZE]>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Accessor {
    Index(usize),
    Range(usize, usize),
    Key(String),
}

impl Value {
    pub fn as_string(&self) -> Option<&str> {
        match self {
            Value::String(s) | Value::Intrinsic(s) => Some(s),
            _ => None,
        }
    }

    pub fn as_list(&self) -> Option<&[Value]> {
        match self {
            Value::List(list) => Some(list),
            _ => None,
        }
    }

    pub fn as_bool(&self) -> Option<bool> {
        match self {
            Value::Boolean(b) => Some(*b),
            _ => None,
        }
    }

    pub fn as_number(&self) -> Option<&Number> {
        match self {
            Value::Number(n) => Some(n),
            _ => None,
        }
    }

    pub fn as_reference(&self) -> Option<&Reference> {
        match self {
            Value::Reference(r) => Some(r),
            _ => None,
        }
    }
}

impl AsRef<str> for Value {
    fn as_ref(&self) -> &str {
        match self {
            Value::String(s) | Value::Intrinsic(s) => s.as_ref(),
            _ => panic!("Attempted to get str reference from non-string Value"),
        }
    }
}

impl AsRef<[Value]> for Value {
    fn as_ref(&self) -> &[Value] {
        match self {
            Value::List(list) => list.as_ref(),
            _ => panic!("Attempted to get list reference from non-list Value"),
        }
    }
}

impl AsRef<bool> for Value {
    fn as_ref(&self) -> &bool {
        match self {
            Value::Boolean(b) => b,
            _ => panic!("Attempted to get bool reference from non-boolean Value"),
        }
    }
}

impl AsRef<Value> for Value {
    fn as_ref(&self) -> &Value {
        self
    }
}
