# Pretty - A Fast JSON Pretty-Printer

A high-performance JSON pretty-printer written in Rust that provides fast formatting with optional colored output.

## Features

- **Fast JSON parsing**: Custom JSON parser optimized for performance
- **Colored output**: Optional ANSI color coding for better readability
- **UTF-8 validation**: Proper handling of UTF-8 encoded JSON
- **Multiple backends**: Choose between custom parser or serde for compatibility
- **Benchmarking**: Built-in iteration support for performance testing
- **BOM handling**: Automatically strips Byte Order Mark from input

## Installation

```bash
cargo build --release
```

The binary will be available at `target/release/pretty`.

### Requirements

- Rust 2024 edition or later
- Cargo package manager

## Usage

### Basic usage

```bash
# Format a JSON file
./pretty input.json

# Format from stdin
echo '{"name":"value"}' | ./pretty -

# Disable colored output
./pretty --no-color input.json

# Use serde backend for compatibility
./pretty --serde input.json

# Run performance benchmarks
./pretty --iter 1000 input.json
```

### Examples

Input:
```json
{"name":"John","age":30,"city":"New York"}
```

Output:
```json
{
  "name": "John",
  "age": 30,
  "city": "New York"
}
```

## Implementation References

### JSON Specification
- [RFC 7159: The JavaScript Object Notation (JSON) Data Interchange Format](https://datatracker.ietf.org/doc/html/rfc7159)

### Related Implementations
- [Go Standard Library JSON Decoder](https://github.com/golang/go/blob/master/src/encoding/json/jsontext/decode.go)
- [Serde JSON](https://github.com/serde-rs/json)

## Test Data Sources

- [Microsoft Edge JSON Demo Data](https://microsoftedge.github.io/Demos/json-dummy-data)
- [JSON Test Suite](https://github.com/nst/JSONTestSuite)
- [UTF-8 Test Suite](https://github.com/flenniken/utf8tests)

## Performance Benchmarks

Performance tests conducted with a 5MB JSON file, running 5000 iterations each.

### Custom Parser vs Serde

**Custom Parser (Release Build)**:
```shell
$ time target/release/pretty --iter 5000 5mb.json > /dev/null 
target/release/pretty --iter 5000 5mb.json  33.77s user 2.57s system 99% cpu 36.392 total
```

**Serde Backend (Release Build)**:
```shell
$ time target/release/pretty --serde --iter 5000 5mb.json > /dev/null
target/release/pretty --serde --iter 5000 5mb.json  69.38s user 10.02s system 99% cpu 1:19.49 total
```

### Parser Evolution Benchmarks

The following benchmarks show performance improvements across different parser implementations:

**Early byte reader implementation**:
```shell
$ time pretty --iter 5000 5mb.json > /dev/null        
pretty --iter 5000 5mb.json  11.25s user 0.86s system 99% cpu 12.134 total
```

**With whitespace skipping**:
```shell
$ time pretty --iter 5000 5mb.json > /dev/null
pretty --iter 5000 5mb.json  35.43s user 1.10s system 98% cpu 36.914 total
```

**With UTF-8 validation before parsing**:
```shell
$ time pretty --iter 5000 5mb.json > /dev/null
pretty --iter 5000 5mb.json  21.16s user 0.05s system 99% cpu 21.248 total
```

**With UTF-8 validation and whitespace skipping**:
```shell
$ time pretty --iter 5000 5mb.json > /dev/null
pretty --iter 5000 5mb.json  43.04s user 0.08s system 99% cpu 43.163 total
```

> **Note**: The custom parser significantly outperforms serde, being approximately 2x faster for large JSON files.

## Development Status

### Completed Features
- [x] Empty array formatting as `[]`
- [x] Nesting level limits
- [x] BOM (Byte Order Mark) removal
- [x] Array-based UTF-8 error handling (using `[]` instead of `Vec` in `InvalidUtf8`)

### Planned Features
- [ ] Unit tests for string parsing
- [ ] Comprehensive JSON parsing test suite
- [ ] Trailing newline management

## License

This project is available under the terms specified in the repository.