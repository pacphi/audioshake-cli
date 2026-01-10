# Developer Guide

This guide is for developers who want to contribute to the AudioShake CLI or build it from source.

## Prerequisites

- **Rust 1.92+**: Install via [rustup](https://rustup.rs/)
- **cloudflared**: Required for local file processing (optional for development)
- **Git**: For version control

## Getting Started

### Clone the Repository

```bash
git clone https://github.com/pacphi/audioshake-cli.git
cd audioshake-cli
```

### Install Development Tools

```bash
make install
```

This installs:

- cargo-nextest (faster test runner)
- cargo-audit (security auditing)
- cargo-llvm-cov (code coverage)

### Build

```bash
# Debug build
make build

# Release build
make build-release
```

### Run Tests

```bash
# Run all tests
make test

# Run with coverage
make test-coverage
```

### Run Linter

```bash
# Check for issues
make lint

# Auto-fix issues
make lint-fix
```

### Format Code

```bash
# Format all code
make format

# Check formatting
make format-check
```

## Project Structure

```text
audioshake-cli/
├── Cargo.toml              # Project manifest
├── Makefile                # Development commands
├── src/
│   ├── main.rs             # Entry point
│   ├── lib.rs              # Library exports
│   ├── cli/                # CLI argument definitions
│   │   ├── mod.rs
│   │   └── args.rs         # Clap argument structs
│   ├── commands/           # Command implementations
│   │   ├── mod.rs
│   │   ├── process.rs      # Process command
│   │   ├── task/           # Task subcommands
│   │   ├── models.rs       # Models command
│   │   ├── config.rs       # Config command
│   │   └── completion.rs   # Shell completions
│   ├── api/                # API client
│   │   ├── mod.rs
│   │   ├── client.rs       # HTTP client
│   │   ├── request.rs      # Request types
│   │   └── response.rs     # Response types
│   ├── config/             # Configuration management
│   │   ├── mod.rs
│   │   ├── loader.rs       # Config loading
│   │   └── types.rs        # Config types
│   ├── tunnel/             # Local file server
│   │   └── mod.rs          # cloudflared tunnel
│   ├── download/           # File downloads
│   │   ├── mod.rs
│   │   └── manager.rs      # Download manager
│   ├── output/             # Output formatting
│   │   └── mod.rs
│   └── error.rs            # Error types
├── tests/                  # Integration tests
├── docs/                   # Documentation
└── .github/                # CI/CD workflows
```

## Architecture

### Command Flow

1. **CLI Parsing**: `src/cli/args.rs` defines all CLI arguments using clap derive macros
2. **Command Dispatch**: `src/main.rs` parses args and dispatches to command handlers
3. **Config Loading**: `src/config/loader.rs` loads config from env, .env, and config file
4. **Command Execution**: Each command in `src/commands/` executes its logic
5. **API Calls**: `src/api/client.rs` handles HTTP communication with AudioShake API

### Key Design Decisions

- **Async Runtime**: Uses tokio for all async operations
- **Error Handling**: Uses thiserror for structured errors with good messages
- **API Key Security**: Uses secrecy crate to prevent accidental logging
- **Local File Support**: Uses axum for local HTTP server + cloudflared tunnel

## Adding a New Command

1. Add argument struct in `src/cli/args.rs`:

```rust
#[derive(Parser, Debug)]
pub struct MyCommandArgs {
    #[arg(short, long)]
    pub option: String,
}
```

2. Add to Commands enum:

```rust
#[derive(Subcommand, Debug)]
pub enum Commands {
    // ...existing commands...
    MyCommand(MyCommandArgs),
}
```

3. Create command handler in `src/commands/mycommand.rs`:

```rust
use crate::cli::args::MyCommandArgs;
use crate::config::types::AppConfig;
use crate::error::Result;

pub async fn execute(args: MyCommandArgs, config: &AppConfig) -> Result<()> {
    // Implementation
    Ok(())
}
```

4. Wire up in `src/main.rs`:

```rust
Commands::MyCommand(args) => commands::mycommand::execute(args, &config).await,
```

## Testing

### Unit Tests

Add tests in the same file as the code:

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_something() {
        // ...
    }

    #[tokio::test]
    async fn test_async_something() {
        // ...
    }
}
```

### Integration Tests

Add tests in `tests/` directory:

```rust
// tests/my_test.rs
use audioshake_cli::api::client::ApiClient;

#[tokio::test]
async fn test_api_client() {
    // Use wiremock for mocking HTTP
}
```

### CLI Tests

Use assert_cmd for CLI testing:

```rust
use assert_cmd::Command;
use predicates::prelude::*;

#[test]
fn test_help_command() {
    Command::cargo_bin("audioshake")
        .unwrap()
        .arg("--help")
        .assert()
        .success()
        .stdout(predicate::str::contains("AudioShake"));
}
```

## CI/CD

### GitHub Actions Workflows

- **ci.yml**: Runs on all PRs and pushes to main
  - Format check
  - Clippy linting
  - Security audit
  - Unit tests
  - Build verification

- **release.yml**: Runs on version tags (v\*)
  - Creates GitHub release
  - Builds binaries for all platforms
  - Publishes to crates.io (if token configured)

### Creating a Release

```bash
# Update version and create tag
make gh-release V=1.0.0
```

This will:

1. Update version in Cargo.toml
2. Create a git commit and tag
3. Push to trigger release workflow

## Code Style

### Rust Guidelines

- Follow [Rust API Guidelines](https://rust-lang.github.io/api-guidelines/)
- Use `#[must_use]` for functions that return values that should be used
- Document public APIs with rustdoc comments
- Prefer returning `Result` over panicking

### Error Handling

- Use the error types defined in `src/error.rs`
- Provide helpful error messages for users
- Include context in error messages

```rust
// Good
Err(AppError::User(format!(
    "File not found: {}. Check the path and try again.",
    path.display()
)))

// Bad
Err(AppError::User("File not found".to_string()))
```

### CLI Output

- Use `console` crate for colored output
- Use `indicatif` for progress bars
- Use `comfy-table` for tabular data
- Respect `--quiet` and `--verbose` flags

## Debugging

### Enable Debug Logging

```bash
RUST_LOG=debug cargo run -- process song.mp3 -m vocals
```

### Log Levels

- `error`: Fatal errors
- `warn`: Warnings
- `info`: Informational messages
- `debug`: Debug information
- `trace`: Very detailed tracing

## Performance Considerations

- Use streaming for large file downloads
- Use parallel downloads with tokio
- Avoid blocking operations in async code
- Use connection pooling (reqwest default)

## Security

- Never log API keys
- Use HTTPS for all API calls
- Validate user input
- Use secrecy crate for sensitive data

## Resources

- [AudioShake API Documentation](https://developer.audioshake.ai/)
- [Rust Book](https://doc.rust-lang.org/book/)
- [Tokio Tutorial](https://tokio.rs/tokio/tutorial)
- [Clap Documentation](https://docs.rs/clap/latest/clap/)
