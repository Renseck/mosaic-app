# MyProject

A Rust project template with best practices and modern tooling.

## Features

- Modern Rust project structure
- CLI with clap
- Configuration management with serde
- Error handling with thiserror and anyhow
- Logging with tracing
- Comprehensive testing setup
- Benchmarking with criterion
- CI/CD with GitHub Actions

## Prerequisites

- Rust 1.90+ (install from [rustup.rs](https://rustup.rs))
- Cargo (comes with Rust)

## Installation

```bash
# Clone the repository
git clone https://github.com/yourusername/myproject.git
cd myproject

# Build the project
cargo build

# Or use make
make build
```

## Configuration

Copy `.env.example` to `.env` and adjust settings:

```bash
cp .env.example .env
```

Create a `config.toml` file for application settings:

```toml
environment = "development"
debug = true

[app]
name = "myproject"
data_dir = "./data"
```

## Usage

```bash
# Run the application
cargo run -- run

# Or using make
make run

# Show help
cargo run -- --help

# Run with verbose logging
cargo run -- -vv run

# Run with input
cargo run -- run --input "test data"

# Show application info
cargo run -- info

# Generate shell completions
cargo run -- completions bash > myproject.bash
```

## Development

```bash
# Run tests
make test

# Run tests with output
make test-verbose

# Run benchmarks
make bench

# Format code
make fmt

# Run linter
make lint

# Check without building
make check

# Generate documentation
make doc

# Watch for changes and run tests
cargo watch -x test
```

## Project Structure

```
src/
  ├── main.rs          # Application entry point
  ├── lib.rs           # Library root
  ├── cli.rs           # CLI implementation
  ├── config/          # Configuration
  ├── core/            # Core logic
  └── error.rs         # Error types

tests/                 # Integration tests
benches/               # Benchmarks
examples/              # Example usage
```

## Building

```bash
# Debug build
cargo build

# Release build (optimized)
cargo build --release

# The binary will be in target/release/myproject
```

## Testing

```bash
# Run all tests
cargo test

# Run specific test
cargo test test_name

# Run tests with output
cargo test -- --nocapture

# Run integration tests only
cargo test --test integration_test
```

## Benchmarking

```bash
# Run benchmarks
cargo bench

# Results will be in target/criterion/
```

## License

MIT License

## Contributing

Contributions welcome! Please open an issue or submit a pull request.