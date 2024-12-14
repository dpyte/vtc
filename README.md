# VTC (Virtual Transport Configuration) Parser and Runtime

VTC is a powerful and flexible configuration language parser and runtime environment implemented in Rust. It provides a robust way to define, manage, and evaluate complex configuration data with support for advanced features like intrinsic functions and dynamic references.

## Features

- Intuitive syntax for defining namespaces and variables
- Support for diverse data types including strings, numbers, booleans, and lists
- Dynamic reference system allowing variables to reference other variables
- Accessor syntax for indexing and slicing lists and strings
- Intrinsic functions for performing operations within the configuration
- High-performance parsing and runtime evaluation
- Extensible library with support for custom functions
- Built-in support for common system operations

## Installation

To use VTC in your Rust project, add the following to your `Cargo.toml`:

```toml
[dependencies]
vtc = { git = "https://github.com/dpyte/vtc.git" }
```

## Usage

Here's a quick example demonstrating the power of VTC:

```rust
use vtc::Runtime;
use std::path::PathBuf;

fn main() -> Result<(), Box<dyn std::error::Error>> {
	let config_path = PathBuf::from("path/to/your/config.vtc");
	let mut runtime = Runtime::from(config_path)?;

	// Optionally update the library loader with custom configurations
	runtime.update_library_loader(YourCustomLoader::configure_default())?;

    let host = runtime.get_string("server_config", "host")?;
    let port = runtime.get_integer("server_config", "port")?;
    let max_threads = runtime.get_integer("server_config", "max_threads")?;
    let timeout = runtime.get_float("server_config", "connection_timeout")?;

    println!("Server Configuration:");
    println!("Host: {}", host);
    println!("Port: {}", port);
    println!("Max Threads: {}", max_threads);
    println!("Connection Timeout: {} seconds", timeout);

    Ok(())
}
```

## Syntax

VTC uses an intuitive syntax for defining configurations:

- Namespaces are defined with `@namespace_name:`
- Variables are defined with `$variable_name := value`
- Lists are enclosed in `[]`
- References use `%` for accessing variables in other namespaces
- Accessors use `->` followed by `(index)` or `(start..end)` for ranges
- Intrinsic functions are called using `[function_name!!, arg1, arg2, ...]`

## Performance

VTC is designed with performance in mind, making it suitable for both small-scale and large-scale configuration management:

- Parsing: Efficient single-pass parsing using the nom library
- Runtime value retrieval: Optimized for quick access and evaluation
- Memory usage: Utilizes reference counting for efficient memory management

## Advanced Features

- **Intrinsic Functions**: Perform calculations and transformations within your configuration.
- **Dynamic References**: Reference and combine values from different parts of your configuration.
- **Type Safety**: Strong typing ensures configuration integrity.
- **Extensibility**: Easily add custom intrinsic functions to suit your specific needs.
- **System Operations**: Built-in support for common system operations.

## Usage Examples

### Using Built-in Standard Functions

VTC comes with a variety of built-in standard functions that you can use in your configurations. Here are some examples:

```
@math_operations:
    $sum := [std_add_int!!, 10, 20]
    $product := [std_mul_int!!, 5, 6]
    $quotient := [std_div_float!!, 10.0, 3.0]

@string_operations:
    $original := "Hello, World!"
    $uppercase := [std_to_uppercase!!, %original]
    $lowercase := [std_to_lowercase!!, %original]
    $substring := [std_substring!!, %original, 0, 5]

@advanced_operations:
    $data := "VTC is awesome"
    $encoded := [std_base64_encode!!, %data]
    $decoded := [std_base64_decode!!, %encoded]
    $hashed := [std_hash!!, %data, "sha256"]

@control_flow:
    $condition := True
    $result := [std_if!!, %condition, "Condition is true", "Condition is false"]
```

In your Rust code, you can access these values like this:

```rust
let sum = runtime.get_integer("math_operations", "sum") ?;
let uppercase = runtime.get_string("string_operations", "uppercase") ?;
let encoded = runtime.get_string("advanced_operations", "encoded") ?;
let result = runtime.get_string("control_flow", "result") ?;

println!("Sum: {}", sum);
println!("Uppercase: {}", uppercase);
println!("Encoded: {}", encoded);
println!("Result: {}", result);
```

### Defining and Using Custom Functions

You can extend VTC's functionality by defining custom functions. Here's an example of how to define and use a custom
function:

First, define your custom function in Rust:

```rust
use vtc::{StdLibLoader, Value, VtcFn};
use std::rc::Rc;

fn custom_multiply_by_two(args: Vec<Rc<Value>>) -> Rc<Value> {
	if args.len() != 1 {
		panic!("custom_multiply_by_two expects 1 argument");
	}

	match &*args[0] {
		Value::Number(Number::Integer(i)) => Arc::new(Value::Number(Number::Integer(i * 2))),
		Value::Number(Number::Float(f)) => Arc::new(Value::Number(Number::Float(f * 2.0))),
		_ => panic!("custom_multiply_by_two expects a number"),
	}
}

impl YourCustomLoader {
	pub fn configure_default() -> StdLibLoader {
		let mut loader = StdLibLoader::new();
		loader.register_function("custom_multiply_by_two".to_string(),
		                         Box::new(custom_multiply_by_two) as VtcFn)
			.unwrap();
		loader
	}
}
```

Then, use this custom function in your VTC configuration:

```
@custom_operations:
    $original := 21
    $doubled := [custom_multiply_by_two!!, %original]
    $doubled_again := [custom_multiply_by_two!!, %doubled]
```

In your Rust code, you can use it like this:

```rust
use vtc::Runtime;
use std::path::PathBuf;

fn main() -> Result<(), Box<dyn std::error::Error>> {
	let config_path = PathBuf::from("path/to/your/config.vtc");
	let mut runtime = Runtime::from(config_path)?;

	// Update the library loader with your custom configurations
	runtime.update_library_loader(YourCustomLoader::configure_default())?;

	let original = runtime.get_integer("custom_operations", "original")?;
	let doubled = runtime.get_integer("custom_operations", "doubled")?;
	let doubled_again = runtime.get_integer("custom_operations", "doubled_again")?;

	println!("Original: {}", original);
	println!("Doubled: {}", doubled);
	println!("Doubled Again: {}", doubled_again);

	Ok(())
}
```

This example demonstrates how to define a custom function `custom_multiply_by_two`, register it with the `StdLibLoader`,
and then use it in your VTC configuration.

## License

This project is licensed under the [MIT License](LICENSE).
