use crate::parser::ast::{Accessor, Reference, ReferenceType, Value, VtcFile};
use crate::parser::grammar::parse;
use crate::parser::lexer::tokenize;
use std::collections::{HashMap, HashSet};
use std::fs;
use std::path::PathBuf;

/// Represents all possible runtime errors.
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
    FileReadError(String),
}

/// Represents the runtime environment.
#[derive(Debug)]
pub struct Runtime {
    namespaces: HashMap<String, HashMap<String, Value>>,
    file: String,
    is_file: bool,
}

impl Runtime {
    /// Creates a new `Runtime` instance.
    pub fn new() -> Self {
        Runtime {
            namespaces: HashMap::new(),
            file: "".to_string(),
            is_file: false,
        }
    }

    /// Reads a file from a given `PathBuf` and processes its content.
    ///
    /// # Arguments
    ///
    /// * `path_buf` - The path to the file to be read.
    ///
    /// # Returns
    ///
    /// * `Result<(), RuntimeError>` - Result type indicating success or the type of runtime error encountered.
    pub fn read_file(&mut self, path_buf: PathBuf) -> Result<(), RuntimeError> {
        let contents = fs::read_to_string(path_buf.clone())
            .map_err(|_| RuntimeError::FileReadError(path_buf.to_str().unwrap().to_string()))?;
        self.file = contents.clone();
        self.is_file = true;

        let result = self.load_vtc(contents.as_str());
        result
    }

    /// Processes the VTC content.
    ///
    /// # Arguments
    ///
    /// * `input` - The input string containing the VTC content.
    ///
    /// # Returns
    ///
    /// * `Result<(), RuntimeError>` - Result type indicating success or the type of runtime error encountered.
    pub fn load_vtc(&mut self, input: &str) -> Result<(), RuntimeError> {
        let (remaining, tokens) = tokenize(input)
            .map_err(|e| RuntimeError::ParseError(format!("Tokenization failed: {:?}", e)))?;
        if !remaining.is_empty() {
            return Err(RuntimeError::ParseError(
                "Input was not fully parsed".to_string(),
            ));
        }

        let vtc_file = parse(&tokens).map_err(|e| RuntimeError::ParseError(e.to_string()))?;
        println!("{:#}", vtc_file);

        let result = self.load_vtc_file(vtc_file);
        result
    }

    /// Loads the VTC file structure into the runtime environment.
    ///
    /// # Arguments
    ///
    /// * `vtc_file` - The `VtcFile` structure to be loaded.
    ///
    /// # Returns
    ///
    /// * `Result<(), RuntimeError>` - Result type indicating success or the type of runtime error encountered.
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

    /// Retrieves a value based on a given reference.
    ///
    /// # Arguments
    ///
    /// * `reference` - The `Reference` to look up the value.
    ///
    /// # Returns
    ///
    /// * `Result<Value, RuntimeError>` - Result type containing the value or the type of runtime error encountered.
    pub fn get_value_with_ref(&self, reference: &Reference) -> Result<Value, RuntimeError> {
        let result = self.get_value_recursive(reference, &mut Vec::new());
        result
    }

    /// Retrieves a value from a given namespace, variable, reference type, and accessors.
    ///
    /// # Arguments
    ///
    /// * `namespace` - The namespace to look up the variable.
    /// * `variable` - The variable name to look up.
    /// * `ref_type` - The type of reference.
    /// * `accessors` - A vector of accessors.
    ///
    /// # Returns
    ///
    /// * `Result<Value, RuntimeError>` - Result type containing the value or the type of runtime error encountered.
    pub fn get_value(
        &self,
        namespace: &str,
        variable: &str,
        ref_type: ReferenceType,
        accessors: Vec<Accessor>,
    ) -> Result<Value, RuntimeError> {
        let reference = Reference {
            ref_type,
            namespace: Some(namespace.to_string()),
            variable: variable.to_string(),
            accessors,
        };
        // let result = self.get_value_with_ref(&reference);
        let result = self.resolve_reference(&reference);
        result
    }

    /// Resolves a reference to retrieve its value.
    ///
    /// # Arguments
    ///
    /// * `reference` - The `Reference` to resolve.
    ///
    /// # Returns
    ///
    /// * `Result<Value, RuntimeError>` - Result type containing the value or the type of runtime error encountered.
    pub fn resolve_reference(&self, reference: &Reference) -> Result<Value, RuntimeError> {
        let mut visited = HashSet::new();
        let result = self.resolve_reference_recursive(reference, &mut visited);
        result
    }

    /// A helper method to recursively retrieve a value from a given reference.
    ///
    /// # Arguments
    ///
    /// * `reference` - The `Reference` to look up the value.
    /// * `visited` - A mutable vector to keep track of visited references.
    ///
    /// # Returns
    ///
    /// * `Result<Value, RuntimeError>` - Result type containing the value or the type of runtime error encountered.
    fn get_value_recursive(
        &self,
        reference: &Reference,
        visited: &mut Vec<(Option<String>, String)>,
    ) -> Result<Value, RuntimeError> {
        let key = (reference.namespace.clone(), reference.variable.clone());
        if visited.contains(&key) {
            return Err(RuntimeError::CircularReference);
        }
        visited.push(key);

        let namespace = match &reference.namespace {
            Some(ns) => ns,
            None => {
                // TODO: Remove this call as the runtime will now be responsible in evaluating this call
                if reference.ref_type == ReferenceType::External {
                    return Err(RuntimeError::MissingNamespace);
                }
                self.namespaces
                    .keys()
                    .next()
                    .ok_or(RuntimeError::NoNamespaces)?
            }
        };

        let variables = self
            .namespaces
            .get(namespace)
            .ok_or(RuntimeError::NamespaceNotFound(namespace.clone()))?;

        let mut value = variables
            .get(&reference.variable)
            .ok_or(RuntimeError::VariableNotFound(reference.variable.clone()))?
            .clone();

        while let Value::Reference(inner_reference) = value {
            value = self.get_value_recursive(&inner_reference, visited)?;
        }

        // Apply accessors from the reference
        for accessor in &reference.accessors {
            value = self.apply_accessor(value, accessor)?;
        }

        Ok(value)
    }

    /// A helper method to recursively resolve a reference.
    ///
    /// # Arguments
    ///
    /// * `reference` - The `Reference` to resolve.
    /// * `visited` - A mutable vector to keep track of visited references.
    ///
    /// # Returns
    ///
    /// * `Result<Value, RuntimeError>` - Result type containing the value or the type of runtime error encountered.
    fn resolve_reference_recursive(
        &self,
        reference: &Reference,
        visited: &mut HashSet<(Option<String>, String)>,
    ) -> Result<Value, RuntimeError> {
        let key = (reference.namespace.clone(), reference.variable.clone());
        if !visited.insert(key.clone()) {
            return Err(RuntimeError::CircularReference);
        }

        let namespace = match &reference.namespace {
            Some(ns) => ns,
            None => {
                if reference.ref_type == ReferenceType::External {
                    return Err(RuntimeError::MissingNamespace);
                }
                self.namespaces
                    .keys()
                    .next()
                    .ok_or(RuntimeError::NoNamespaces)?
            }
        };

        let variables = self.namespaces.get(namespace).ok_or_else(|| {
            RuntimeError::NamespaceNotFound(format!("Unable to locate namespace `{:?}`", namespace))
        })?;

        let mut value = variables
            .get(&reference.variable)
            .ok_or_else(|| RuntimeError::VariableNotFound(reference.variable.clone()))?
            .clone();

        // Resolve nested references
        value = self.resolve_value(value, visited)?;

        // Apply accessors
        for accessor in &reference.accessors {
            value = self.apply_accessor(value, accessor)?;
        }

        visited.remove(&key);
        Ok(value)
    }

    /// Resolves a value, handling nested references.
    ///
    /// # Arguments
    ///
    /// * `value` - The value to resolve.
    /// * `visited` - A mutable vector to keep track of visited references.
    ///
    /// # Returns
    ///
    /// * `Result<Value, RuntimeError>` - Result type containing the resolved value or the type of runtime error encountered.
    fn resolve_value(
        &self,
        value: Value,
        visited: &mut HashSet<(Option<String>, String)>,
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

    /// Applies an accessor to a given value.
    ///
    /// # Arguments
    ///
    /// * `value` - The value to apply the accessor to.
    /// * `accessor` - The accessor to apply.
    ///
    /// # Returns
    ///
    /// * `Result<Value, RuntimeError>` - Result type containing the modified value or the type of runtime error encountered.
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
