# Contributing to LayKit

Thank you for your interest in contributing to LayKit! This guide will help you get started.

## Getting Started

### Prerequisites

- Rust 1.70 or later
- Git
- A GitHub account
- Familiarity with Rust and Git workflows

### Setting Up Development Environment

1. **Fork the repository**
   ```bash
   # Fork on GitHub, then clone your fork
   git clone https://github.com/YOUR_USERNAME/laykit.git
   cd laykit
   ```

2. **Add upstream remote**
   ```bash
   git remote add upstream https://github.com/giridharsalana/laykit.git
   ```

3. **Install Rust (if needed)**
   ```bash
   curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
   ```

4. **Build and test**
   ```bash
   cargo build
   cargo test
   cargo clippy
   cargo fmt --check
   ```

## Development Workflow

### 1. Create a Branch

```bash
git checkout -b feature/your-feature-name
# or
git checkout -b fix/bug-description
```

**Branch naming conventions:**
- `feature/` - New features
- `fix/` - Bug fixes
- `docs/` - Documentation changes
- `refactor/` - Code refactoring
- `test/` - Test additions/changes

### 2. Make Changes

Follow the coding standards and write tests for your changes.

### 3. Test Your Changes

```bash
# Run tests
cargo test

# Run tests with output
cargo test -- --nocapture

# Check formatting
cargo fmt --check

# Run clippy
cargo clippy -- -D warnings

# Build documentation
cargo doc --open
```

### 4. Commit Changes

```bash
git add .
git commit -m "feat: add new feature description"
```

**Commit message format:**
```
type: brief description

Longer description if needed.

Fixes #issue_number
```

**Types:**
- `feat:` - New feature
- `fix:` - Bug fix
- `docs:` - Documentation
- `test:` - Tests
- `refactor:` - Code refactoring
- `perf:` - Performance improvement
- `chore:` - Maintenance

### 5. Push and Create PR

```bash
git push origin feature/your-feature-name
```

Then create a Pull Request on GitHub.

## Code Style

### Rust Style

Follow Rust standard style using `rustfmt`:

```bash
cargo fmt
```

**Key points:**
- Use 4-space indentation
- Line length: 100 characters (preferred)
- Use trailing commas in multi-line constructs
- Organize imports: std, external crates, internal modules

**Example:**
```rust
use std::collections::HashMap;
use std::error::Error;
use std::fs::File;

use laykit::{GDSIIFile, OASISFile};

pub fn process_file(path: &str) -> Result<(), Box<dyn Error>> {
    let gds = GDSIIFile::read_from_file(path)?;
    // Process...
    Ok(())
}
```

### Documentation

Document all public items:

```rust
/// Reads a GDSII file from the specified path.
///
/// # Arguments
///
/// * `path` - Path to the GDSII file
///
/// # Returns
///
/// Returns a `Result` containing the `GDSIIFile` or an error.
///
/// # Examples
///
/// ```
/// use laykit::GDSIIFile;
///
/// let gds = GDSIIFile::read_from_file("design.gds")?;
/// # Ok::<(), Box<dyn std::error::Error>>(())
/// ```
pub fn read_from_file(path: &str) -> Result<Self, Box<dyn Error>> {
    // Implementation...
}
```

### Error Handling

Use `Result` types consistently:

```rust
// Good
pub fn process() -> Result<Output, Box<dyn Error>> {
    let data = read_file()?;
    Ok(process_data(data))
}

// Avoid unwrap in library code
// Bad
pub fn process() -> Output {
    let data = read_file().unwrap(); // Don't do this!
    process_data(data)
}
```

## Testing

### Writing Tests

Every new feature should have tests:

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_feature() -> Result<(), Box<dyn Error>> {
        // Setup
        let input = create_test_input();
        
        // Execute
        let result = new_feature(input)?;
        
        // Verify
        assert_eq!(result.value, expected_value);
        
        Ok(())
    }
}
```

### Test Coverage

Aim for:
- ✅ 80%+ code coverage for new features
- ✅ Tests for all public APIs
- ✅ Tests for error conditions
- ✅ Integration tests for complex features

## Pull Request Guidelines

### Before Submitting

Checklist:
- ✅ Code compiles without warnings
- ✅ All tests pass
- ✅ `cargo clippy` passes
- ✅ `cargo fmt` applied
- ✅ Documentation updated
- ✅ CHANGELOG updated (if applicable)
- ✅ New tests added

### PR Description

Include in your PR:

```markdown
## Description
Brief description of changes

## Type of Change
- [ ] Bug fix
- [ ] New feature
- [ ] Breaking change
- [ ] Documentation update

## Testing
How has this been tested?

## Checklist
- [ ] Tests pass locally
- [ ] Code follows project style
- [ ] Documentation updated
- [ ] CHANGELOG updated
```

### PR Review Process

1. Automated checks run (CI/CD)
2. Code review by maintainers
3. Address feedback
4. Approval and merge

## Areas for Contribution

### High Priority

- **Performance optimizations** - SIMD, parallel processing
- **Streaming API** - For very large files
- **Additional element types** - Missing GDSII/OASIS features
- **CLI tool** - Command-line interface
- **More tests** - Edge cases, error conditions

### Good First Issues

Look for issues labeled:
- `good first issue`
- `help wanted`
- `documentation`

### Documentation

- Improve API documentation
- Add more examples
- Write tutorials
- Fix typos and clarify instructions

### Testing

- Add edge case tests
- Improve test coverage
- Add benchmark tests
- Add integration tests

## Code Review

### What We Look For

- **Correctness** - Does it work?
- **Tests** - Are there adequate tests?
- **Style** - Follows project conventions?
- **Documentation** - Well documented?
- **Performance** - No obvious performance issues?
- **Safety** - No unsafe code without justification?

### Responding to Feedback

- Be open to suggestions
- Ask questions if unclear
- Make requested changes
- Push updates to the same branch

## Release Process

(For maintainers)

1. Update version in `Cargo.toml`
2. Update `CHANGELOG.md`
3. Create git tag: `git tag v0.x.y`
4. Push tag: `git push origin v0.x.y`
5. CI builds and deploys automatically

## Communication

### Channels

- **GitHub Issues** - Bug reports, feature requests
- **GitHub Discussions** - Questions, ideas, general discussion
- **Pull Requests** - Code contributions

### Asking Questions

Before asking:
1. Check existing issues
2. Read the documentation
3. Search discussions

When asking:
- Provide context
- Include code examples
- Show what you've tried
- Be specific about the problem

## License

By contributing, you agree that your contributions will be licensed under the MIT License.

## Code of Conduct

### Our Standards

- Be respectful and inclusive
- Accept constructive criticism gracefully
- Focus on what's best for the community
- Show empathy towards others

### Unacceptable Behavior

- Harassment or discriminatory language
- Trolling or insulting comments
- Publishing others' private information
- Other unprofessional conduct

## Getting Help

If you need help:

1. Check the [documentation](https://giridharsalana.github.io/laykit/)
2. Search [existing issues](https://github.com/giridharsalana/laykit/issues)
3. Ask in [discussions](https://github.com/giridharsalana/laykit/discussions)
4. Open a new issue

## Recognition

Contributors will be:
- Listed in release notes
- Credited in documentation
- Mentioned in the project README

Thank you for contributing to LayKit! 🎉
