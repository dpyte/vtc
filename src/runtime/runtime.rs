use crate::parser::ast::{Accessor, Reference, ReferenceType, Value, VtcFile};
use crate::parser::grammar::parse;
use crate::parser::lexer::tokenize;
use std::collections::HashMap;

#[derive(Debug)]
pub enum RuntimeError {
    CircularReference,
    IndexOutOfBounds(usize),
    InvalidAccessor(String),
    InvalidRange(usize, usize),
    MissingNamespace,
    NamespaceNotFound(String),
    NoNamespaces,
    ParseError(String),
    VariableNotFound(String),
}

#[derive(Debug)]
pub struct Runtime {
    namespaces: HashMap<String, HashMap<String, Value>>,
}

impl Runtime {
    pub fn new() -> Self {
        Runtime {
            namespaces: HashMap::new(),
        }
    }

    pub fn load_vtc(&mut self, input: &str) -> Result<(), RuntimeError> {
        let (remaining, tokens) = tokenize(input).map_err(|e| RuntimeError::ParseError(format!("Tokenization failed: {:?}", e)))?;
        if !remaining.is_empty() {
            return Err(RuntimeError::ParseError("Input was not fully parsed".to_string()));
        }
        let (_, vtc_file) = parse(&tokens).map_err(|e| RuntimeError::ParseError(format!("Parsing failed: {:?}", e)))?;
        self.load_vtc_file(vtc_file)
    }

    pub fn load_vtc_file(&mut self, vtc_file: VtcFile) -> Result<(), RuntimeError> {
        for namespace in vtc_file.namespaces {
            let mut variables = HashMap::new();
            for variable in namespace.variables {
                variables.insert(variable.name.clone(), variable.value.clone());
            }
            self.namespaces.insert(namespace.name.clone(), variables);
        }
        Ok(())
    }

    pub fn get_value_with_ref(&self, reference: &Reference) -> Result<Value, RuntimeError> {
        let result = self.get_value_recursive(reference, &mut Vec::new());
        result
    }

    pub fn get_value(&self, namespace: &str, variable: &str, ref_type: ReferenceType, accessors: Vec<Accessor>) -> Result<Value, RuntimeError> {
        let reference = Reference {
            ref_type,
            namespace: Some(namespace.to_string()),
            variable: variable.to_string(),
            accessors,
        };
        let result = self.get_value_with_ref(&reference);
        result
    }


    // pub fn get_value(&self, reference: &Reference) -> Result<Value, RuntimeError> {
    //     let namespace = match &reference.namespace {
    //         Some(ns) => ns,
    //         None => {
    //             if reference.ref_type == ReferenceType::External {
    //                 return Err(RuntimeError::MissingNamespace);
    //             }
    //             // For local references, use the first namespace (assuming there's only one)
    //             self.namespaces.keys().next().ok_or(RuntimeError::NoNamespaces)?
    //         }
    //     };
    //
    //     let variables = self.namespaces.get(namespace).ok_or(RuntimeError::NamespaceNotFound(namespace.clone()))?;
    //     let mut value = variables.get(&reference.variable).ok_or(RuntimeError::VariableNotFound(reference.variable.clone()))?.clone();
    //
    //     for accessor in &reference.accessors {
    //         value = self.apply_accessor(value, accessor)?;
    //     }
    //
    //     Ok(value)
    // }
    fn get_value_recursive(&self, reference: &Reference, visited: &mut Vec<(Option<String>, String)>) -> Result<Value, RuntimeError> {
        let key = (reference.namespace.clone(), reference.variable.clone());
        if visited.contains(&key) {
            return Err(RuntimeError::CircularReference);
        }
        visited.push(key);

        let namespace = match &reference.namespace {
            Some(ns) => ns,
            None => {
                if reference.ref_type == ReferenceType::External {
                    return Err(RuntimeError::MissingNamespace);
                }
                self.namespaces.keys().next().ok_or(RuntimeError::NoNamespaces)?
            }
        };

        let variables = self.namespaces.get(namespace).ok_or(RuntimeError::NamespaceNotFound(namespace.clone()))?;
        let mut value = variables.get(&reference.variable).ok_or(RuntimeError::VariableNotFound(reference.variable.clone()))?.clone();

        // If the value is a reference, resolve it
        if let Value::Reference(inner_reference) = value {
            value = self.get_value_recursive(&inner_reference, visited)?;
        }

        // Apply accessors from the reference
        for accessor in &reference.accessors {
            value = self.apply_accessor(value, accessor)?;
        }

        Ok(value)
    }

    fn apply_accessor(&self, value: Value, accessor: &Accessor) -> Result<Value, RuntimeError> {
        match (value, accessor) {
            (Value::List(list), Accessor::Index(index)) => {
                list.get(*index).cloned().ok_or(RuntimeError::IndexOutOfBounds(*index))
            }
            (Value::String(s), Accessor::Index(index)) => {
                s.chars().nth(*index)
                    .map(|c| Value::String(c.to_string()))
                    .ok_or(RuntimeError::IndexOutOfBounds(*index))
            }
            (Value::List(list), Accessor::Range(start, end)) => {
                if *start >= list.len() || *end > list.len() || start > end {
                    return Err(RuntimeError::InvalidRange(*start, *end));
                }
                Ok(Value::List(list[*start..*end].to_vec()))
            }
            (Value::Reference(ref inner_reference), _) => {
                let resolved_value = self.get_value_recursive(inner_reference, &mut Vec::new())?;
                self.apply_accessor(resolved_value, accessor)
            }
            (_, Accessor::Key(key)) => {
                Err(RuntimeError::InvalidAccessor(format!("Key accessor '{}' not supported for this value type", key)))
            }
            _ => Err(RuntimeError::InvalidAccessor("Accessor not supported for this value type".to_string())),
        }
    }
}
