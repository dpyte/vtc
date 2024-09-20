#[derive(Debug, PartialEq, Clone)]
pub struct VtcFile {
    pub namespaces: Vec<Namespace>,
}

#[derive(Debug, PartialEq, Clone)]
pub struct Namespace {
    pub name: String,
    pub variables: Vec<Variable>,
}

#[derive(Debug, PartialEq, Clone)]
pub struct Variable {
    pub name: String,
    pub value: Value,
}

#[derive(Debug, PartialEq, Clone)]
pub enum Value {
    String(String),
    Number(Number),
    Boolean(bool),
    Nil,
    List(Vec<Value>),
    Reference(Reference),
}

#[derive(Debug, PartialEq, Clone)]
pub enum Number {
    Integer(i64),
    Float(f64),
    Binary(i64),
    Hexadecimal(i64),
}

#[derive(Debug, PartialEq, Clone)]
pub enum  ReferenceType {
    External, // &
    Local, // %
}

#[derive(Debug, PartialEq, Clone)]
pub struct Reference {
    pub ref_type: ReferenceType,
    pub namespace: Option<String>,
    pub variable: String,
    pub accessors: Vec<Accessor>,
}

#[derive(Debug, PartialEq, Clone)]
pub enum Accessor {
    Index(usize),
    Range(usize, usize),
    Key(String),
}