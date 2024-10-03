# VTC (Virtual Transport Configuration) Parser and Runtime

VTC is a powerful and flexible configuration language parser and runtime environment implemented in Rust. It provides a robust way to define, manage, and evaluate complex configuration data with support for advanced features like intrinsic functions and dynamic references.

## Features

- Intuitive syntax for defining namespaces and variables
- Support for diverse data types including strings, numbers, booleans, and lists
- Dynamic reference system allowing variables to reference other variables
- Accessor syntax for indexing and slicing lists and strings
- Intrinsic functions for performing operations within the configuration
- High-performance parsing and runtime evaluation
- Comprehensive error handling and reporting

## Installation

To use VTC in your Rust project, add the following to your `Cargo.toml`:

```toml
[dependencies]
vtc = { git = "https://github.com/yourusername/vtc.git" }
```

## Usage

Here's a quick example demonstrating the power of VTC:

```rust
use vtc::Runtime;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let input = r#"
    @network_config:
        $ports := [80, 443, 8080]
        $enabled_protocols := ["http", "https", "websocket"]
        $max_connections := 1000
        $timeout_ms := 30000

    @server_config:
        $host := "example.com"
        $port := %network_config.ports->(0)
        $protocol := %network_config.enabled_protocols->(1)
        $max_threads := [std_mul_int!!, %network_config.max_connections, 2]
        $connection_timeout := [std_div_float!!, %network_config.timeout_ms, 1000.0]
    "#;

    let mut runtime = Runtime::new();
    runtime.load_vtc(input)?;

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

For more detailed syntax information, refer to the [documentation](link_to_docs).

## Performance

VTC is designed with performance in mind, making it suitable for both small-scale and large-scale configuration management:

- Parsing: Efficient single-pass parsing
- Runtime value retrieval: Optimized for quick access and evaluation
- Memory usage: Utilizes reference counting for efficient memory management

## Advanced Features

- **Intrinsic Functions**: Perform calculations and transformations within your configuration.
- **Dynamic References**: Reference and combine values from different parts of your configuration.
- **Type Safety**: Strong typing ensures configuration integrity.
- **Extensibility**: Easily add custom intrinsic functions to suit your specific needs.

## Contributing

Contributions are welcome! Whether it's bug reports, feature requests, or code contributions, please feel free to engage with the project. Check out our [CONTRIBUTING.md](link_to_contributing) for guidelines.

## License

This project is licensed under the [MIT License](LICENSE).
