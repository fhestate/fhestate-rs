# 🤝 Contributing to FHESTATE

**Help us build the future of privacy on Solana.**

[![FHESTATE](https://img.shields.io/badge/FHESTATE-Community-8A2BE2?style=for-the-badge&logo=rocket&logoColor=white)](https://github.com/fhestate/fhestate-rs)
[![Code of Conduct](https://img.shields.io/badge/Contributor-Covenant-2E8B57?style=for-the-badge&logo=handshake&logoColor=white)](CODE_OF_CONDUCT.md)

---

## Contribution Navigator

*   **1. Getting Started**
    *   [Setup Environment](#getting-started)
    *   [Development Workflow](#development-workflow)
*   **2. Standards**
    *   [Coding Style](#coding-standards)
    *   [Testing Guidelines](#testing)
*   **3. Submission**
    *   [Pull Request Process](#pull-request-process)
    *   [Areas for Contribution](#areas-for-contribution)

---

## Code of Conduct

### Our Pledge

We are committed to providing a welcoming and inclusive environment for all contributors.

### Our Standards

**Positive behavior includes:**
- Using welcoming and inclusive language
- Being respectful of differing viewpoints
- Gracefully accepting constructive criticism
- Focusing on what is best for the community

**Unacceptable behavior includes:**
- Harassment of any kind
- Trolling, insulting comments, or personal attacks
- Publishing others' private information
- Other conduct that would be considered inappropriate

---

## Getting Started

### Prerequisites

- Rust 1.70+
- Solana CLI 1.18+
- Git
- Basic understanding of FHE concepts

# Fork the repository on GitHub
# Then clone your fork
git clone https://github.com/FHESTATE/fhestate-rs.git
cd fhestate-rs

# Add upstream remote
git remote add upstream https://github.com/fhestate/fhestate-rs.git

### Build and Test

```bash
# Build the project
cargo build

# Run tests
cargo test

# Run linter
cargo clippy -- -D warnings

# Format code
cargo fmt -- --check
```

---

## Development Workflow

### 1. Create a Branch

```bash
git checkout -b feature/your-feature-name
```

**Branch naming conventions:**
- `feature/` - New features
- `fix/` - Bug fixes
- `docs/` - Documentation changes
- `refactor/` - Code refactoring
- `test/` - Test additions/changes

### 2. Make Changes

- Write clear, concise code
- Add tests for new features
- Update documentation as needed
- Follow coding standards (below)

### 3. Commit

```bash
git add .
git commit -m "feat: add homomorphic multiplication support"
```

**Commit message format:**
```
<type>: <description>

[optional body]
[optional footer]
```

**Types:**
- `feat`: New feature
- `fix`: Bug fix
- `docs`: Documentation only
- `style`: Formatting changes
- `refactor`: Code restructuring
- `test`: Adding tests
- `chore`: Maintenance tasks

### 4. Push and Create PR

```bash
git push origin feature/your-feature-name
```

Then create a Pull Request on GitHub.

---

## Coding Standards

### Rust Style

Follow the official [Rust Style Guide](https://rust-lang.github.io/api-guidelines/):

```rust
// Good: descriptive names, proper formatting
pub fn encrypt_value(
    plaintext: u8,
    client_key: &ClientKey
) -> Result<FheUint8, FheError> {
    if plaintext > 255 {
        return Err(FheError::InvalidInput);
    }
    
    Ok(FheUint8::encrypt(plaintext, client_key))
}

// Bad: unclear names, poor formatting
pub fn enc(p: u8, k: &ClientKey) -> FheUint8 {FheUint8::encrypt(p, k)}
```

### Documentation

All public items must have documentation:

```rust
/// Encrypts a plaintext value using the given client key.
///
/// # Arguments
///
/// * `plaintext` - The value to encrypt (0-255)
/// * `client_key` - The FHE client key for encryption
///
/// # Returns
///
/// Encrypted ciphertext as `FheUint8`
///
/// # Examples
///
/// ```
/// let ct = encrypt_value(42, &client_key)?;
/// ```
pub fn encrypt_value(
    plaintext: u8,
    client_key: &ClientKey
) -> FheUint8 {
    FheUint8::encrypt(plaintext, client_key)
}
```

### Error Handling

Use `Result` for fallible operations:

```rust
// Good
pub fn load_key(path: &str) -> Result<ClientKey, FheError> {
    let bytes = std::fs::read(path)
        .map_err(|e| FheError::KeyNotFound(e.to_string()))?;
    
    bincode::deserialize(&bytes)
        .map_err(|e| FheError::DeserializationError(e.to_string()))
}

// Bad
pub fn load_key(path: &str) -> ClientKey {
    let bytes = std::fs::read(path).unwrap();
    bincode::deserialize(&bytes).unwrap()
}
```

---

## Testing

### Unit Tests

```rust
#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_encryption_decryption() {
        let (client_key, _) = generate_test_keys();
        let plaintext = 42;
        
        let ciphertext = FheUint8::encrypt(plaintext, &client_key);
        let decrypted: u8 = ciphertext.decrypt(&client_key);
        
        assert_eq!(plaintext, decrypted);
    }
}
```

### Integration Tests

```rust
// tests/integration_test.rs
use fhestate_sdk::*;

#[test]
fn test_full_workflow() {
    // Generate keys
    let (client_key, server_key) = generate_keys_default();
    
    // Encrypt
    let ct = FheUint8::encrypt(10, &client_key);
    
    // Compute
    set_server_key(server_key);
    let result = ct + FheUint8::encrypt(32, &client_key);
    
    // Decrypt
    let plaintext: u8 = result.decrypt(&client_key);
    assert_eq!(plaintext, 42);
}
```

### Running Tests

```bash
# All tests
cargo test

# Specific test
cargo test test_encryption_decryption

# With output
cargo test -- --nocapture

# Integration tests only
cargo test --test '*'
```

---

## Pull Request Process

### Before Submitting

- [ ] Code compiles without errors
- [ ] All tests pass
- [ ] Code is formatted (`cargo fmt`)
- [ ] No clippy warnings (`cargo clippy`)
- [ ] Documentation is updated
- [ ] Commit messages follow convention

### PR Description Template

```markdown
## Description
Brief description of changes

## Type of Change
- [ ] Bug fix
- [ ] New feature
- [ ] Breaking change
- [ ] Documentation update

## Testing
How was this tested?

## Checklist
- [ ] My code follows the project style
- [ ] I have added tests
- [ ] I have updated documentation
- [ ] All tests pass
```

### Review Process

1. Automated checks run (CI/CD)
2. Maintainer reviews code
3. Address feedback
4. Approval and merge

---

## Areas for Contribution

### High Priority

- **Custom Solana Program**: Deploy coordinator program
- **Active Executor**: Implement `fhe-node` polling
- **Error Handling**: Improve error messages
- **Documentation**: More examples and tutorials

### 1. Project Naming
For clarity, use the following naming conventions in code and documentation:
- **fhestate-rs**: The core Rust SDK package.
- **fhe-node**: The background compute service.
- **fhe-cli**: The command-line interface for Solana interaction.

### Low Priority

- **Examples**: More use cases
- **Benchmarks**: Performance comparisons
- **Integrations**: Libraries for other languages

---

## Questions?

- **Discussion**: [GitHub Discussions](https://github.com/fhestate/fhestate-sdk/discussions)
- **Twitter**: [Join our twitter](https://x.com/fhe_state)

---
