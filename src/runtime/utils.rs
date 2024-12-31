use std::sync::Arc;

use fnv::FnvHashMap;

use crate::runtime::error::RuntimeError;
use crate::runtime::Runtime;
use crate::value::{Number, Value};

impl Runtime {
    pub fn get_string_value(&self, value: &Arc<Value>) -> Result<String, RuntimeError> {
        if let Value::String(s) = &**value {
            Ok(s.to_string())
        } else {
            Err(RuntimeError::TypeError("Expected string".to_string()))
        }
    }

    pub fn get_string(&self, namespace: &str, variable: &str) -> Result<String, RuntimeError> {
        let value = self.get_value(namespace, variable, &[])?;
        self.get_string_value(&value)
    }

    pub fn to_string(&self, value: Arc<Value>) -> Result<String, RuntimeError> {
        let str_value = value.to_string();
        if str_value.len() < 2 {
            return Ok(str_value);
        }
        Ok(str_value[1..str_value.len() - 1].to_string())
    }

    /// Generic function to get and type check values
    fn get_typed_value<T, F>(
        &self,
        namespace: &str,
        variable: &str,
        f: F,
    ) -> Result<T, RuntimeError>
    where
        F: FnOnce(&Arc<Value>) -> Result<T, RuntimeError>,
    {
        let value = self.get_value(namespace, variable, &[])?;
        f(&value)
    }

    pub fn get_integer(&self, namespace: &str, variable: &str) -> Result<i64, RuntimeError> {
        self.get_typed_value(namespace, variable, |v| match &**v {
            Value::Number(Number::Integer(i)) => Ok(*i),
            Value::Number(Number::Binary(b)) => Ok(*b),
            Value::Number(Number::Hexadecimal(h)) => Ok(*h),
            _ => Err(RuntimeError::TypeError("Expected integer".to_string())),
        })
    }

    pub fn get_float(&self, namespace: &str, variable: &str) -> Result<f64, RuntimeError> {
        self.get_typed_value(namespace, variable, |v| {
            if let Value::Number(Number::Float(f)) = &**v {
                Ok(*f)
            } else {
                Err(RuntimeError::TypeError("Expected float".to_string()))
            }
        })
    }

    pub fn get_boolean(&self, namespace: &str, variable: &str) -> Result<bool, RuntimeError> {
        self.get_typed_value(namespace, variable, |v| {
            if let Value::Boolean(b) = &**v {
                Ok(*b)
            } else {
                Err(RuntimeError::TypeError("Expected boolean".to_string()))
            }
        })
    }

    pub fn get_list(&self, namespace: &str, variable: &str) -> Result<Vec<Value>, RuntimeError> {
        self.get_typed_value(namespace, variable, |v| {
            if let Value::List(l) = &**v {
                Ok(l.iter().map(|value| (*value).clone()).collect::<Vec<_>>())
            } else {
                Err(RuntimeError::TypeError("Expected list".to_string()))
            }
        })
    }

    /// Convert list to dictionary.
    /// Constraints: The size of list must be in multiples of 2.

    pub fn as_dict(
        &self,
        namespace: &str,
        variable: &str,
    ) -> Result<FnvHashMap<String, Arc<Value>>, RuntimeError> {
        let values = self.get_list(namespace, variable)?;
        let mut result = FnvHashMap::default();

        for chunk in values.chunks(2) {
            if chunk.len() != 2 {
                return Err(RuntimeError::ConversionError(format!(
                    "List size must be even in {}::{}",
                    namespace, variable
                )));
            }

            let key = match &chunk[0] {
                Value::String(s) => s.clone(),
                _ => {
                    return Err(RuntimeError::ConversionError(format!(
                        "Key must be a string in {}::{}",
                        namespace, variable
                    )))
                }
            };

            result.insert(key, Arc::new(chunk[1].clone()));
        }

        Ok(result)
    }

    /// flattens a nested list to a single dimension list
    pub fn flatten_list(
        &self,
        namespace: &str,
        variable: &str,
    ) -> Result<Vec<Value>, RuntimeError> {
        let list = self.get_list(namespace, variable)?;
        let mut flattened = Vec::with_capacity(list.len());

        fn flatten_recursive(value: &Value, output: &mut Vec<Value>) {
            match value {
                Value::List(items) => {
                    for item in items.iter() {
                        flatten_recursive(item, output);
                    }
                }
                _ => output.push(value.clone()),
            }
        }

        for value in list {
            flatten_recursive(&value, &mut flattened);
        }

        Ok(flattened)
    }
}
