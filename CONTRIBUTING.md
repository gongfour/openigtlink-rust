# Contributing to OpenIGTLink Rust

Thank you for your interest in contributing to OpenIGTLink Rust! This document provides guidelines and instructions for developing and contributing to the project.

## Table of Contents

- [Development Setup](#development-setup)
- [Code Style](#code-style)
- [Running Tests](#running-tests)
- [Submitting Changes](#submitting-changes)
- [Release Process](#release-process)

## Development Setup

### Prerequisites

- Rust 1.70.0 or later (stable toolchain)
- Cargo (comes with Rust)

### Getting Started

1. Fork and clone the repository:
   ```bash
   git clone https://github.com/your-username/openigtlink-rust.git
   cd openigtlink-rust
   ```

2. Build the project:
   ```bash
   cargo build
   ```

3. Run tests to verify your setup:
   ```bash
   cargo test
   ```

## Code Style

This project follows the standard Rust style guidelines enforced by `rustfmt` and `clippy`.

### Before Committing

**Always run these commands before committing:**

```bash
# Format your code
cargo fmt

# Check for common mistakes and improvements
cargo clippy

# Run all tests
cargo test
```

### Automated Formatting

To avoid forgetting to format your code, you can set up automatic formatting:

#### Option 1: Pre-commit Hook

Create `.git/hooks/pre-commit`:

```bash
#!/bin/sh
cargo fmt --all
cargo clippy -- -D warnings
```

Make it executable:
```bash
chmod +x .git/hooks/pre-commit
```

#### Option 2: Editor Integration

**VS Code** - Add to `.vscode/settings.json`:
```json
{
  "editor.formatOnSave": true,
  "[rust]": {
    "editor.defaultFormatter": "rust-lang.rust-analyzer"
  }
}
```

**Other editors**: Most Rust plugins support format-on-save with rust-analyzer.

### CI Checks

Our CI pipeline automatically checks:
- ✅ Code formatting (`cargo fmt -- --check`)
- ✅ Clippy warnings (`cargo clippy -- -D warnings`)
- ✅ All tests pass (`cargo test`)
- ✅ Documentation builds (`cargo doc`)

**Your PR will fail CI if any of these checks fail.**

## Running Tests

### Basic Tests

```bash
# Run all tests
cargo test

# Run tests with output
cargo test -- --nocapture

# Run a specific test
cargo test test_name
```

### Integration Tests

```bash
# Run only integration tests
cargo test --test '*'

# Run examples (requires OpenIGTLink server)
cargo run --example async_client
cargo run --example client
cargo run --example server
```

### Documentation Tests

```bash
# Test code examples in documentation
cargo test --doc
```

### Benchmarks

```bash
# Run benchmarks (requires nightly)
cargo +nightly bench
```

## Submitting Changes

### Pull Request Process

1. **Create a feature branch:**
   ```bash
   git checkout -b feature/your-feature-name
   ```

2. **Make your changes and commit:**
   ```bash
   # Format and lint
   cargo fmt
   cargo clippy

   # Test thoroughly
   cargo test

   # Commit with a descriptive message
   git commit -m "Add feature: description"
   ```

3. **Push to your fork:**
   ```bash
   git push origin feature/your-feature-name
   ```

4. **Open a Pull Request:**
   - Provide a clear description of your changes
   - Reference any related issues
   - Ensure CI checks pass

### Commit Message Guidelines

Follow conventional commit format:

```
type(scope): brief description

Detailed explanation if needed
```

**Types:**
- `feat`: New feature
- `fix`: Bug fix
- `docs`: Documentation changes
- `style`: Code style changes (formatting, etc.)
- `refactor`: Code refactoring
- `test`: Adding or updating tests
- `chore`: Maintenance tasks

**Examples:**
```
feat(client): add support for TCP keepalive
fix(parser): handle empty device names correctly
docs(readme): update installation instructions
test(integration): add async client timeout tests
```

## Release Process

This section is for maintainers.

### Version Bump

1. Update version in `Cargo.toml`
2. Update `CHANGELOG.md`
3. Commit changes:
   ```bash
   git commit -m "chore: bump version to X.Y.Z"
   ```
4. Create and push tag:
   ```bash
   git tag vX.Y.Z
   git push origin main --tags
   ```

The CI/CD pipeline will automatically:
- Run all validation checks
- Publish to crates.io
- Create a GitHub release

## Getting Help

- 📖 Check the [documentation](https://docs.rs/openigtlink-rust)
- 💬 Open an [issue](https://github.com/gongfour/openigtlink-rust/issues) for bugs or questions
- 📧 Contact the maintainers

## Code of Conduct

- Be respectful and inclusive
- Focus on constructive feedback
- Help others learn and grow

Thank you for contributing to OpenIGTLink Rust! 🦀
