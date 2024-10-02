use std::collections::{HashMap, HashSet};
use std::fs;
use std::path::PathBuf;
use crate::parser::ast::{Accessor, Number, Reference, ReferenceType, Value, VtcFile};
use crate::parser::grammar::parse;
use crate::parser::lexer::tokenize;
use crate::runtime::error::RuntimeError;

/// Represents the runtime environment for VTC files.
#[derive(Debug)]
pub struct Runtime {
    namespaces: HashMap<String, HashMap<String, Value>>,
}

impl Runtime {
    /// Creates a new `Runtime` instance.
    pub fn new() -> Self {
        Runtime {
            namespaces: HashMap::new(),
        }
    }

    /// Reads a VTC file from the given path and loads it into the runtime.
    ///
    /// # Arguments
    ///
    /// * `path` - The path to the VTC file.
    ///
    /// # Returns
    ///
    /// * `Result<(), RuntimeError>` - Ok if successful, or an error if file reading or parsing fails.
    pub fn load_file(&mut self, path: PathBuf) -> Result<(), RuntimeError> {
        let contents = fs::read_to_string(&path)
            .map_err(|_| RuntimeError::FileReadError(path.to_str().unwrap().to_string()))?;
        self.load_vtc(&contents)
    }

    /// Loads VTC content from a string into the runtime.
    ///
    /// # Arguments
    ///
    /// * `input` - The VTC content as a string.
    ///
    /// # Returns
    ///
    /// * `Result<(), RuntimeError>` - Ok if successful, or an error if parsing fails.
    pub fn load_vtc(&mut self, input: &str) -> Result<(), RuntimeError> {
        let vtc_file = self.parse_vtc(input)?;
        self.load_vtc_file(vtc_file)
    }

    // Private methods for parsing and loading

    fn parse_vtc(&self, input: &str) -> Result<VtcFile, RuntimeError> {
        let (remaining, tokens) = tokenize(input)
            .map_err(|e| RuntimeError::ParseError(format!("Tokenization failed: {:?}", e)))?;

        if !remaining.is_empty() {
            return Err(RuntimeError::ParseError("Input was not fully parsed".to_string()));
        }

        parse(&tokens).map_err(|e| RuntimeError::ParseError(e.to_string()))
    }

    fn load_vtc_file(&mut self, vtc_file: VtcFile) -> Result<(), RuntimeError> {
        for namespace in vtc_file.namespaces {
            let variables: HashMap<String, Value> = namespace.variables
                .into_iter()
                .map(|var| (var.name.clone(), var.value.clone()))
                .collect();
            self.namespaces.insert(namespace.name.clone(), variables);
        }
        Ok(())
    }

    // Public methods for value retrieval

    /// Retrieves a value from the specified namespace and variable.
    ///
    /// # Arguments
    ///
    /// * `namespace` - The namespace containing the variable.
    /// * `variable` - The name of the variable.
    /// * `accessors` - Optional accessors to apply to the value.
    ///
    /// # Returns
    ///
    /// * `Result<Value, RuntimeError>` - The retrieved value or an error if not found.
    pub fn get_value(&self, namespace: &str, variable: &str, accessors: &[Accessor]) -> Result<Value, RuntimeError> {
        let reference = Reference {
            ref_type: ReferenceType::Local,
            namespace: Some(namespace.to_string()),
            variable: variable.to_string(),
            accessors: accessors.to_vec(),
        };
        self.resolve_reference(&reference)
    }

    /// Converts a Value to its string representation.
    ///
    /// # Arguments
    ///
    /// * `value` - The Value to convert.
    ///
    /// # Returns
    ///
    /// * `Result<String, RuntimeError>` - The string representation or an error.
    pub fn to_string(&self, value: Value) -> Result<String, RuntimeError> {
        let mut result = value.to_string();
        result = result[1..result.len()-1].to_string();
        Ok(result)
    }

    // Type-specific value retrieval methods

    /// Retrieves a string value from the specified namespace and variable.
    pub fn get_string(&self, namespace: &str, variable: &str) -> Result<String, RuntimeError> {
        self.get_typed_value(namespace, variable, |v| {
            if let Value::String(s) = v {
                Ok(s.clone())
            } else {
                Err(RuntimeError::TypeError("Expected string".to_string()))
            }
        })
    }

    /// Retrieves an integer value from the specified namespace and variable.
    pub fn get_integer(&self, namespace: &str, variable: &str) -> Result<i64, RuntimeError> {
        self.get_typed_value(namespace, variable, |v| {
            if let Value::Number(Number::Integer(i)) = v {
                Ok(i.clone())
            } else {
                Err(RuntimeError::TypeError("Expected integer".to_string()))
            }
        })
    }

    /// Retrieves a float value from the specified namespace and variable.
    pub fn get_float(&self, namespace: &str, variable: &str) -> Result<f64, RuntimeError> {
        self.get_typed_value(namespace, variable, |v| {
            if let Value::Number(Number::Float(f)) = v {
                Ok(f.clone())
            } else {
                Err(RuntimeError::TypeError("Expected float".to_string()))
            }
        })
    }

    /// Retrieves a boolean value from the specified namespace and variable.
    pub fn get_boolean(&self, namespace: &str, variable: &str) -> Result<bool, RuntimeError> {
        self.get_typed_value(namespace, variable, |v| {
            if let Value::Boolean(b) = v {
                Ok(*b)
            } else {
                Err(RuntimeError::TypeError("Expected boolean".to_string()))
            }
        })
    }

    /// Retrieves a list value from the specified namespace and variable.
    pub fn get_list(&self, namespace: &str, variable: &str) -> Result<Vec<Value>, RuntimeError> {
        self.get_typed_value(namespace, variable, |v| {
            if let Value::List(l) = v {
                Ok(l.clone())
            } else {
                Err(RuntimeError::TypeError("Expected list".to_string()))
            }
        })
    }

    // Helper method for type-specific value retrieval
    fn get_typed_value<T, F>(&self, namespace: &str, variable: &str, f: F) -> Result<T, RuntimeError>
    where
        F: FnOnce(&Value) -> Result<T, RuntimeError>,
    {
        let value = self.get_value(namespace, variable, &[])?;
        f(&value)
    }

    // Methods for exploring the runtime content

    /// Lists all namespaces in the runtime.
    pub fn list_namespaces(&self) -> Vec<&String> {
        self.namespaces.keys().collect()
    }

    /// Lists all variables in a specific namespace.
    pub fn list_variables(&self, namespace: &str) -> Result<Vec<&String>, RuntimeError> {
        self.namespaces.get(namespace)
            .map(|vars| vars.keys().collect())
            .ok_or_else(|| RuntimeError::NamespaceNotFound(namespace.to_string()))
    }

    // Private methods for reference resolution

    fn resolve_reference(&self, reference: &Reference) -> Result<Value, RuntimeError> {
        let mut visited = HashSet::new();
        self.resolve_reference_recursive(reference, &mut visited)
    }

    fn resolve_reference_recursive(
        &self,
        reference: &Reference,
        visited: &mut HashSet<(String, String)>,
    ) -> Result<Value, RuntimeError> {
        let namespace = reference.namespace.as_ref()
            .ok_or_else(|| RuntimeError::MissingNamespace)?;
        let key = (namespace.clone(), reference.variable.clone());

        if !visited.insert(key.clone()) {
            return Err(RuntimeError::CircularReference);
        }

        let variables = self.namespaces.get(namespace)
            .ok_or_else(|| RuntimeError::NamespaceNotFound(namespace.clone()))?;

        let mut value = variables.get(&reference.variable)
            .ok_or_else(|| RuntimeError::VariableNotFound(reference.variable.clone()))?
            .clone();

        value = self.resolve_value(value, visited)?;

        for accessor in &reference.accessors {
            value = self.apply_accessor(value, accessor)?;
        }

        visited.remove(&key);
        Ok(value)
    }

    fn resolve_value(
        &self,
        value: Value,
        visited: &mut HashSet<(String, String)>,
    ) -> Result<Value, RuntimeError> {
        match value {
            Value::Reference(inner_reference) => {
                self.resolve_reference_recursive(&inner_reference, visited)
            }
            Value::List(items) => {
                let resolved_items = items
                    .into_iter()
                    .map(|item| self.resolve_value(item, visited))
                    .collect::<Result<Vec<_>, _>>()?;
                Ok(Value::List(resolved_items))
            }
            _ => Ok(value),
        }
    }

    fn apply_accessor(&self, value: Value, accessor: &Accessor) -> Result<Value, RuntimeError> {
        match (value, accessor) {
            (Value::List(list), Accessor::Index(index)) => list
                .get(*index)
                .cloned()
                .ok_or(RuntimeError::IndexOutOfBounds(*index)),
            (Value::List(list), Accessor::Range(start, end)) => {
                if *start > *end || *end > list.len() {
                    Err(RuntimeError::InvalidRange(*start, *end))
                } else {
                    Ok(Value::List(list[*start..*end].to_vec()))
                }
            }
            (Value::String(s), Accessor::Index(index)) => s
                .chars()
                .nth(*index)
                .map(|c| Value::String(c.to_string()))
                .ok_or(RuntimeError::IndexOutOfBounds(*index)),
            (Value::String(s), Accessor::Range(start, end)) => {
                if *start > *end || *end > s.len() {
                    Err(RuntimeError::InvalidRange(*start, *end))
                } else {
                    Ok(Value::String(s[*start..*end].to_string()))
                }
            }
            (_, Accessor::Key(key)) => Err(RuntimeError::InvalidAccessor(format!(
                "Key accessor '{}' not supported for this value type",
                key
            ))),
            _ => Err(RuntimeError::InvalidAccessor(
                "Accessor not supported for this value type".to_string(),
            )),
        }
    }
}