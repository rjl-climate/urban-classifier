# Claude Code Configuration for Rust Development

This document defines the principles and practices for developing production-quality Rust applications using Rust 2024 edition. These guidelines prioritize simplicity, clarity, and maintainability.

## Table of Contents
1. [Core Philosophy](#core-philosophy)
2. [Project Structure](#project-structure)
3. [Code Style Guidelines](#code-style-guidelines)
4. [Error Handling](#error-handling)
5. [Testing Strategy](#testing-strategy)
6. [Async Programming](#async-programming)
7. [Memory Management](#memory-management)
8. [Dependencies](#dependencies)
9. [Documentation](#documentation)
10. [Build & CI/CD](#build--cicd)

## Core Philosophy

### Simplicity Over Cleverness
Write code that is easy to understand, not code that shows off Rust's advanced features.

```rust
// GOOD: Clear and simple
pub fn find_user(id: u64, users: &[User]) -> Option<&User> {
    users.iter().find(|u| u.id == id)
}

// AVOID: Unnecessarily complex
pub fn find_user<'a, I>(id: u64, users: I) -> Option<&'a User>
where
    I: IntoIterator<Item = &'a User>,
{
    users.into_iter().find(|u| u.id == id)
}
```

### Explicit Error Handling
Errors are values. Handle them explicitly and provide context.

```rust
// GOOD: Clear error handling
pub fn read_config(path: &Path) -> Result<Config, ConfigError> {
    let contents = std::fs::read_to_string(path)
        .map_err(|e| ConfigError::ReadFile(path.to_owned(), e))?;
    
    serde_json::from_str(&contents)
        .map_err(|e| ConfigError::ParseJson(e))
}
```

### Zero-Cost Abstractions
Use Rust's type system to enforce correctness without runtime overhead.

## Project Structure

### Standard Layout
```
project-name/
├── Cargo.toml
├── Cargo.lock
├── README.md
├── src/
│   ├── main.rs          # Binary entry point
│   ├── lib.rs           # Library root (if dual binary/library)
│   ├── config.rs        # Configuration handling
│   ├── error.rs         # Error types
│   └── models/          # Domain models
│       └── mod.rs
├── tests/               # Integration tests
│   └── integration.rs
├── benches/             # Benchmarks
│   └── performance.rs
└── examples/            # Example usage
    └── basic.rs
```

### Module Organization
- One concept per module
- Keep files under 300 lines
- Public API at the top of files
- Related functionality grouped together

## Code Style Guidelines

### Naming Conventions
```rust
// Types: PascalCase
struct UserAccount { }
enum Status { }

// Functions and variables: snake_case
fn process_data() { }
let user_name = "Alice";

// Constants: SCREAMING_SNAKE_CASE
const MAX_CONNECTIONS: usize = 100;

// Lifetimes: short, lowercase
fn parse<'a>(input: &'a str) -> &'a str { }
```

### Use Statements
```rust
// Group by: std, external crates, internal modules
use std::collections::HashMap;
use std::fs;

use serde::{Deserialize, Serialize};
use tokio::task;

use crate::config::Config;
use crate::models::User;
```

### Type Annotations
Be explicit when it improves readability:

```rust
// GOOD: Clear what we're collecting
let users: Vec<User> = database
    .fetch_all()
    .await?;

// GOOD: Type inference is obvious
let count = users.len();
```

## Error Handling

### Library Errors
Use `thiserror` for libraries:

```rust
use thiserror::Error;

#[derive(Error, Debug)]
pub enum AppError {
    #[error("Configuration error: {0}")]
    Config(String),
    
    #[error("Database connection failed")]
    Database(#[from] sqlx::Error),
    
    #[error("Invalid user input: {message}")]
    Validation { message: String },
}
```

### Application Errors
Use `anyhow` for applications:

```rust
use anyhow::{Context, Result};

fn main() -> Result<()> {
    let config = load_config()
        .context("Failed to load configuration")?;
    
    run_app(config)
        .context("Application failed")?;
    
    Ok(())
}
```

### Error Design Principles
- Make illegal states unrepresentable
- Provide actionable error messages
- Include relevant context
- Never panic in libraries

## Testing Strategy

### Unit Tests
Keep tests close to the code:

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_user_creation() {
        let user = User::new("Alice", "alice@example.com");
        assert_eq!(user.name(), "Alice");
        assert_eq!(user.email(), "alice@example.com");
    }
}
```

### Integration Tests
```rust
// tests/api_integration.rs
use your_app::start_server;

#[tokio::test]
async fn test_health_endpoint() {
    let app = start_server(test_config()).await;
    
    let response = reqwest::get("http://localhost:8080/health")
        .await
        .expect("Failed to send request");
    
    assert_eq!(response.status(), 200);
}
```

### Test Helpers
```rust
#[cfg(test)]
mod test_helpers {
    use tempfile::TempDir;
    
    pub struct TestContext {
        pub temp_dir: TempDir,
        pub config: Config,
    }
    
    impl TestContext {
        pub fn new() -> Self {
            let temp_dir = TempDir::new().unwrap();
            let config = Config::test_default();
            
            Self { temp_dir, config }
        }
    }
}
```

## Async Programming

### Async Functions
Use async/await with clear patterns:

```rust
use tokio::time::{timeout, Duration};

pub async fn fetch_with_timeout(url: &str) -> Result<String> {
    let duration = Duration::from_secs(30);
    
    timeout(duration, async {
        reqwest::get(url)
            .await?
            .text()
            .await
    })
    .await
    .map_err(|_| anyhow::anyhow!("Request timed out"))?
}
```

### Concurrent Tasks
Use structured concurrency:

```rust
use futures::future::join_all;

pub async fn process_items(items: Vec<Item>) -> Result<Vec<Output>> {
    let tasks: Vec<_> = items
        .into_iter()
        .map(|item| tokio::spawn(async move {
            process_single_item(item).await
        }))
        .collect();
    
    let results = join_all(tasks).await;
    
    results
        .into_iter()
        .collect::<Result<Vec<_>, _>>()?
        .into_iter()
        .collect()
}
```

### Async Traits (Rust 2024)
```rust
// Rust 2024 supports async fn in traits
pub trait DataStore {
    async fn fetch(&self, id: u64) -> Result<Data>;
    async fn store(&self, data: &Data) -> Result<()>;
}
```

## Memory Management

### Ownership Patterns
Design clear ownership from the start:

```rust
// GOOD: Clear ownership
pub struct App {
    config: Config,
    database: Database,
}

impl App {
    pub fn new(config: Config) -> Self {
        let database = Database::connect(&config.database_url);
        Self { config, database }
    }
}
```

### Smart Pointers
Use appropriately:
- `Box<T>` - Heap allocation
- `Rc<T>` - Shared ownership (single-threaded)
- `Arc<T>` - Shared ownership (thread-safe)
- `RefCell<T>` - Interior mutability (single-threaded)
- `Mutex<T>` / `RwLock<T>` - Interior mutability (thread-safe)

### Lifetimes
Keep lifetimes simple:

```rust
// GOOD: Clear lifetime relationship
pub struct Parser<'a> {
    input: &'a str,
}

impl<'a> Parser<'a> {
    pub fn new(input: &'a str) -> Self {
        Self { input }
    }
    
    pub fn parse(&self) -> Result<Token<'a>> {
        // Parse logic
    }
}
```

## Dependencies

### Cargo.toml Best Practices
```toml
[package]
name = "my-app"
version = "0.1.0"
edition = "2024"
rust-version = "1.75"

[dependencies]
# Pin to minor versions for stability
tokio = { version = "1.35", features = ["full"] }
serde = { version = "1.0", features = ["derive"] }
anyhow = "1.0"

[dev-dependencies]
criterion = "0.5"
proptest = "1.4"

[profile.release]
lto = true
codegen-units = 1
```

### Recommended Crates
- **Error Handling**: `thiserror` (libraries), `anyhow` (applications)
- **Serialization**: `serde` with `serde_json`
- **Async Runtime**: `tokio`
- **HTTP**: `reqwest` (client), `axum` (server)
- **CLI**: `clap`
- **Logging**: `tracing`
- **Testing**: `proptest`, `criterion`

## Documentation

### Function Documentation
```rust
/// Processes user data and returns a formatted result.
///
/// # Arguments
///
/// * `user` - The user to process
/// * `options` - Processing options
///
/// # Returns
///
/// Returns `Ok(ProcessedData)` on success, or an error if processing fails.
///
/// # Examples
///
/// ```
/// let user = User::new("Alice", "alice@example.com");
/// let result = process_user(&user, &Default::default())?;
/// assert_eq!(result.status, Status::Completed);
/// ```
pub fn process_user(user: &User, options: &Options) -> Result<ProcessedData> {
    // Implementation
}
```

### Module Documentation
```rust
//! # User Management Module
//!
//! This module provides functionality for managing users in the system.
//!
//! ## Features
//!
//! - User creation and validation
//! - Authentication and authorization
//! - Profile management
```

## Build & CI/CD

### GitHub Actions Workflow
```yaml
name: Rust CI

on:
  push:
    branches: [main]
  pull_request:

env:
  CARGO_TERM_COLOR: always
  RUSTFLAGS: "-D warnings"

jobs:
  test:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@stable
        with:
          components: rustfmt, clippy
      
      - uses: Swatinem/rust-cache@v2
      
      - name: Format
        run: cargo fmt --all -- --check
      
      - name: Lint
        run: cargo clippy --all-targets --all-features
      
      - name: Test
        run: cargo test --all-features
      
      - name: Build
        run: cargo build --release
```

### Pre-commit Hook
```bash
#!/bin/sh
# .git/hooks/pre-commit

set -e

echo "Running pre-commit checks..."

# Format
cargo fmt --all

# Clippy
cargo clippy --all-targets --all-features -- -D warnings

# Test
cargo test

echo "All checks passed!"
```

## Additional Guidelines

### Performance
- Profile before optimizing
- Use `cargo bench` for benchmarks
- Consider `const fn` for compile-time computation
- Prefer iterators over explicit loops

### Security
- Run `cargo audit` regularly
- Keep dependencies updated
- Validate all inputs
- Use safe Rust - minimize `unsafe` blocks

### Debugging
- Use `dbg!()` macro for quick debugging
- Add `#[derive(Debug)]` to all types
- Use `tracing` for production logging
- Consider `cargo expand` for macro debugging

When writing Rust code, prioritize clarity and correctness. The borrow checker is your friend - work with it, not against it. Simple, well-documented code is always preferable to clever, complex solutions.