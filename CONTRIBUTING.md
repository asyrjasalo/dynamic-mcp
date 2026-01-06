# Contributing to dynamic-mcp

Thank you for your interest in contributing to dynamic-mcp! This document provides guidelines and instructions for contributors.

## 🚀 Getting Started

### Prerequisites

- Rust 1.75 or higher
- Cargo
- Git

### Development Setup

1. **Clone the repository**
   ```bash
   git clone https://github.com/yourusername/dynamic-mcp.git
   cd dynamic-mcp
   ```

2. **Install Rust** (if not already installed)
   ```bash
   curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
   ```

3. **Build the project**
   ```bash
   cargo build
   ```

4. **Run tests to verify setup**
   ```bash
   cargo test
   ```

## 🔨 Development Workflow

### Building

```bash
# Debug build (faster compilation, slower execution)
cargo build

# Release build (optimized)
cargo build --release

# Run without building binary
cargo run -- config.example.json
```

### Running Tests

```bash
# Run all tests
cargo test

# Run specific module tests
cargo test config::

# Run with output (see println! statements)
cargo test -- --nocapture

# Run integration tests only
cargo test --test integration_test

# Run specific test
cargo test test_substitute_env_vars

# Run with test coverage
cargo test -- --test-threads=1
```

### Code Structure

- **config/**: Configuration loading, validation, and environment variable substitution
- **proxy/**: MCP client management, group state tracking, transport creation
- **server/**: MCP server that exposes the two-tool API
- **cli/**: Command-line interface and migration tools
- **auth/**: OAuth2 authentication and token management

### Running in Development

```bash
# With command line argument
cargo run -- config.example.json

# With environment variable
export GATEWAY_MCP_CONFIG=config.example.json
cargo run

# With debug logging
RUST_LOG=debug cargo run -- config.example.json
```

## 🧪 Testing Guidelines

### Test Categories

1. **Unit Tests**: Located alongside source code in `src/`
2. **Integration Tests**: Located in `tests/` directory

### Writing Tests

- Place unit tests in the same file as the code they test
- Use `#[cfg(test)]` module for test code
- Use descriptive test names that explain what is being tested
- Test both success and failure cases
- Mock external dependencies where appropriate

Example:
```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_env_var_substitution() {
        // Arrange
        std::env::set_var("TEST_VAR", "test_value");
        
        // Act
        let result = substitute_env_vars("${TEST_VAR}");
        
        // Assert
        assert_eq!(result, "test_value");
    }
}
```

## 📋 Code Style

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

## 📝 Commit Guidelines

### Commit Message Format

Use conventional commits format:

```
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

**Examples:**
```
feat(auth): add OAuth2 token refresh support

Implements automatic token refresh before expiry using refresh tokens.
Tokens are stored in ~/.dynamic-mcp/oauth-servers/

Closes #123
```

```
fix(config): handle missing environment variables

Properly validate and warn when environment variables are undefined
instead of silently failing.
```

## 🔄 Pull Request Process

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

- [ ] Code follows project style guidelines
- [ ] All tests pass (`cargo test`)
- [ ] New tests added for new functionality
- [ ] Documentation updated (if applicable)
- [ ] Commit messages follow conventions
- [ ] No compiler warnings
- [ ] `cargo fmt` and `cargo clippy` pass

## 🐛 Reporting Bugs

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

## 💡 Suggesting Features

Feature suggestions are welcome! Please:

1. Check existing issues to avoid duplicates
2. Clearly describe the feature and its benefits
3. Provide use cases and examples
4. Be open to discussion and feedback

## 📚 Additional Resources

- **[docs/implementation/DEVELOPMENT.md](docs/implementation/DEVELOPMENT.md)** - Project status, roadmap, and metrics
- **[AGENTS.md](AGENTS.md)** - Guidelines for AI coding agents (documentation maintenance)
- **[Architecture](docs/ARCHITECTURE.md)** - System design and component details
- **[Implementation Docs](docs/implementation/)** - Detailed implementation documentation
- **[Rust Book](https://doc.rust-lang.org/book/)** - Official Rust learning resource
- **[MCP Specification](https://modelcontextprotocol.io/)** - Model Context Protocol docs

## 🤝 Code of Conduct

- Be respectful and constructive
- Welcome newcomers and help them get started
- Focus on what's best for the project and community
- Accept constructive criticism gracefully

## 📄 License

By contributing to dynamic-mcp, you agree that your contributions will be licensed under the MIT License.

---

Thank you for contributing to dynamic-mcp! 🎉
