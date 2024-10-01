use std::fmt;

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
pub enum ReferenceType {
    External, // &
    Local,    // %
}

#[derive(Debug, PartialEq, Clone)]
pub struct Reference {
    pub ref_type: ReferenceType,
    pub namespace: Option<String>,
    pub variable: String,
    pub accessors: Vec<Accessor>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Accessor {
    Index(usize),
    Range(usize, usize),
    Key(String),
}

impl fmt::Display for VtcFile {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for namespace in &self.namespaces {
            write!(f, "{}", namespace)?;
            if namespace != self.namespaces.last().unwrap() {
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
            if variable != self.variables.last().unwrap() {
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
