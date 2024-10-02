# VTC (Variable Type Configuration) Parser and Runtime

VTC is a custom configuration language parser and runtime environment implemented in Rust. It provides a flexible way to define and manage configuration data with support for complex data structures and references.

## Features

- Custom syntax for defining namespaces and variables
- Support for various data types including strings, numbers, booleans, and lists
- Reference system allowing variables to reference other variables
- Accessor syntax for indexing and slicing lists and strings
- Robust error handling and reporting
- High-performance parsing and runtime evaluation

## Installation

To use VTC in your Rust project, add the following to your `Cargo.toml`:

```toml
[dependencies]
vtc = { git = "https://github.com/yourusername/vtc.git" }
```

## Usage

Here's a quick example of how to use VTC:

```rust
use vtc::Runtime;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let input = r#"
    @test_sample:
        $value_1 := ["hello", "world", "\0"]
        $value_2 := [True, False, %test_sample.value_1->(0..2), False, "Hello", 1]
    "#;

    let mut runtime = Runtime::new();
    runtime.load_vtc(input)?;

    let value = runtime.get_value("test_sample", "value_2", vtc::ReferenceType::Local, vec![])?;
    println!("Value: {:?}", value);

    Ok(())
}
```

## Syntax

VTC uses a custom syntax for defining configuration:

- Namespaces are defined with `@namespace_name:`
- Variables are defined with `$variable_name := value`
- Lists are enclosed in `[]`
- References use `&` for external references and `%` for local references
- Accessors use `->` followed by `(index)` or `(start..end)` for ranges

For more detailed syntax information, refer to the [documentation](link_to_docs).

## Performance

VTC is designed with performance in mind. Here are some benchmark results:

- Tokenization: TBD 
- Parsing: TBD 
- Runtime value retrieval: ~1.05 µs

## Contributing

Contributions are welcome! Please feel free to submit a Pull Request.

## License

This project is licensed under the [MIT License](LICENSE).
