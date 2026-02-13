# Contributing to dynamic-mcp

Thank you for your interest in contributing to dynamic-mcp! This document provides guidelines and instructions for contributors.

## Getting Started

### Prerequisites

- Rust 1.75 or higher
- Cargo
- Git
- Python 3.9+ (for Python package development)
- Maturin (for building Python wheels)

### Development Setup

1. **Clone the repository**

   ```bash
   git clone https://github.com/asyrjasalo/dynamic-mcp.git
   cd dynamic-mcp
   ```

2. **Install Rust** (if not already installed)

   ```bash
   curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
   ```

3. **Install Python tools** (optional, for Python package work)

   ```bash
   pip install maturin build
   ```

4. **Install pre-commit hooks** (recommended)

   ```bash
   prek install --hook-type pre-commit --hook-type commit-msg
   ```

5. **Build the project**

   ```bash
   cargo build
   ```

6. **Run tests to verify setup**

   ```bash
   cargo test
   ```

## Development Workflow

### Building

```bash
# Debug build (faster compilation, slower execution)
cargo build

# Release build (optimized)
cargo build --release

# Run without building binary
cargo run -- examples/config.example.json

# With environment variable
export DYNAMIC_MCP_CONFIG=examples/config.example.json
cargo run

# With debug logging
RUST_LOG=debug cargo run -- examples/config.example.json
```

### Running Tests

```bash
# Run all tests
cargo test

# Run with output visible
cargo test -- --nocapture

# Run single-threaded (for debugging)
cargo test -- --test-threads=1
```

**For detailed test organization, coverage, and commands, see [docs/implementation/TESTING.md](docs/implementation/TESTING.md).**

### Running Benchmarks

```bash
# Run performance benchmarks
cargo bench --bench performance
```

### Python Package Development

For working on the Python package:

```bash
# Build Python wheel
maturin build --release

# Install locally for testing
pip install --force-reinstall target/wheels/dmcp-*.whl

# Test with uvx (without installing)
uvx --from target/wheels/dmcp-*.whl dmcp --help

# Run with config
uvx --from target/wheels/dmcp-*.whl dmcp config.json

# Build for all platforms (requires cross-compilation setup)
# See .github/workflows/release.yml for CI configuration
```

**Note**: The Python package uses maturin with `bindings = "bin"` mode, which:

- Compiles the Rust binary directly into the wheel
- Auto-generates the `dmcp` entry point
- Creates platform-specific wheels (one per OS/architecture)
- Requires no Python wrapper code

## Testing Guidelines

**For comprehensive testing documentation, see [docs/implementation/TESTING.md](docs/implementation/TESTING.md).**

### Writing Tests

- Place unit tests in the same file as the code (`#[cfg(test)]` module)
- Place integration tests in `tests/` directory
- Use descriptive test names: `test_<feature>_<scenario>_<expected_result>()`
- Test both success and failure cases
- See [TESTING.md](docs/implementation/TESTING.md) for examples and structure

## Code Style

### Formatting

We use `rustfmt` for consistent code formatting:

```bash
# Format all code
cargo fmt

# Check formatting without changing files
cargo fmt -- --check
```

### Linting

We use `clippy` for catching common mistakes:

```bash
# Run clippy
cargo clippy

# Run clippy with stricter checks
cargo clippy -- -D warnings
```

### Best Practices

- Follow Rust naming conventions (snake_case for functions, CamelCase for types)
- Add documentation comments (`///`) for public APIs
- Keep functions focused and small
- Use descriptive variable names
- Avoid unwrap() in production code; prefer proper error handling
- Write tests for new features and bug fixes

## Commit Guidelines

### Pre-commit Hooks

This project uses [prek](https://github.com/j178/prek) to enforce code quality and consistency.

**Installation:**

```bash
prek install --hook-type pre-commit --hook-type commit-msg
```

**If a hook fails:**

- Many hooks auto-fix issues (trailing whitespace, end-of-file, etc.)
- Review changes with `git diff`
- Stage fixed files with `git add`
- Commit again

### Commit Message Format

Use conventional commits format (enforced by commitizen hook):

```text
<type>(<scope>): <subject>

<body>

<footer>
```

**Types:**

- `feat`: New feature
- `fix`: Bug fix
- `docs`: Documentation changes
- `test`: Adding or updating tests
- `refactor`: Code refactoring
- `perf`: Performance improvements
- `chore`: Maintenance tasks
- `build`: Build system changes

**Examples:**

```text
feat(auth): add OAuth2 token refresh support

Implements automatic token refresh before expiry using refresh tokens.
Tokens are stored in ~/.dynamic-mcp/oauth-servers/

Closes #123
```

```text
fix(config): handle missing environment variables

Properly validate and warn when environment variables are undefined
instead of silently failing.
```

## Pull Request Process

1. **Create a feature branch**

   ```bash
   git checkout -b feature/your-feature-name
   ```

2. **Make your changes**

   - Write clear, focused commits
   - Add tests for new functionality
   - Update documentation as needed

3. **Ensure all checks pass**

   ```bash
   cargo fmt -- --check
   cargo clippy -- -D warnings
   cargo test
   pre-commit run --all-files  # Optional: run pre-commit hooks manually
   ```

4. **Push your branch**

   ```bash
   git push origin feature/your-feature-name
   ```

5. **Open a Pull Request**

   - Provide a clear description of changes
   - Reference any related issues
   - Wait for review and address feedback

### PR Checklist

- [ ] New tests added for new functionality
- [ ] All tests pass (`cargo test`)
- [ ] `cargo fmt` and `cargo clippy` pass
- [ ] No compiler warnings
- [ ] Code follows project style guidelines
- [ ] Documentation is updated
- [ ] Pre-commit hooks installed and passing
- [ ] Commit messages follow conventional commits format

## Reporting Bugs

When reporting bugs, please include:

1. **Description**: Clear description of the issue
2. **Steps to Reproduce**: Minimal steps to reproduce the behavior
3. **Expected Behavior**: What you expected to happen
4. **Actual Behavior**: What actually happened
5. **Environment**:
   - OS and version
   - Rust version (`rustc --version`)
   - dynamic-mcp version
6. **Configuration**: Relevant config file (sanitized)
7. **Logs**: Output with `RUST_LOG=debug` if applicable

## Suggesting Features

Feature suggestions are welcome! Please:

1. Check existing issues to avoid duplicates
2. Clearly describe the feature and its benefits
3. Provide use cases and examples
4. Be open to discussion and feedback

## Additional Resources

### Project Documentation

- **[README.md](README.md)** - User guide and project overview
- **[docs/IMPORT.md](docs/IMPORT.md)** - How to import configs from other AI tools
- **[CLAUDE.md](CLAUDE.md)** - Guidelines for AI coding agents working on this project

### Understanding the Codebase

- **[docs/implementation/ARCHITECTURE.md](docs/implementation/ARCHITECTURE.md)** - System design and component structure
- **[docs/implementation/STATUS.md](docs/implementation/STATUS.md)** - Current features and implementation metrics
- **[docs/implementation/TESTING.md](docs/implementation/TESTING.md)** - Test organization and coverage details
- **[docs/implementation/MCP_SPEC_COMPLIANCE.md](docs/implementation/MCP_SPEC_COMPLIANCE.md)** - How we comply with MCP specification

### External Resources

- **[Rust Book](https://doc.rust-lang.org/book/)** - Official Rust learning resource
- **[MCP Specification](https://modelcontextprotocol.io/)** - Model Context Protocol documentation

## License

By contributing to dynamic-mcp, you agree that your contributions will be licensed under the MIT License.

**_****_****_****_****_****_****_****_****_****_****_****_****_****_**

Thank you for contributing to dynamic-mcp! 🎉
