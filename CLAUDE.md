# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

AudioShake CLI is a Rust command-line interface for the AudioShake API, providing audio/video processing capabilities including stem separation, transcription, and dialogue extraction.

## Build & Development Commands

```bash
# Setup
make install              # Install dev tools (cargo-nextest, cargo-audit, cargo-llvm-cov)

# Build
make build                # Debug build
make build-release        # Release build (LTO optimized)

# Run
make run ARGS='...'       # Run with arguments, e.g., ARGS='process song.mp3 -m vocals'
cargo run -- --help       # Direct cargo run
```

## Testing

```bash
make test                 # Run unit tests (uses cargo-nextest)
make test-integration     # Run integration tests
make test-coverage        # Generate coverage report to coverage/html/index.html

# Run a single test
cargo nextest run test_name
cargo nextest run -p audioshake-cli test_name
```

## Code Quality

```bash
make lint                 # Clippy with pedantic warnings as errors
make lint-fix             # Auto-fix clippy issues
make format               # Format with rustfmt
make format-check         # Check formatting only
```

## Architecture

### Entry Point & CLI Flow
- `src/main.rs` - Tokio async runtime, CLI parsing, command routing
- `src/cli/args.rs` - Clap argument definitions, Commands enum

### Core Modules
- `src/api/` - HTTP client (`client.rs`), request/response types
- `src/commands/` - Command implementations (process, task/*, models, config, completion)
- `src/config/` - Configuration loading from env, .env, TOML file
- `src/tunnel/` - Local file serving via Axum + cloudflared tunnel
- `src/download/` - Result file download orchestration
- `src/error.rs` - Structured error types with thiserror

### Key Design Patterns
- Async-first using Tokio
- API key handling via `secrecy` crate (prevents accidental logging)
- Configuration priority: CLI flag > env var > .env > config file
- Error chain: `AppError` wraps `ConfigError`, `ApiError`, etc.

### Dependencies to Know
- `clap` for CLI parsing
- `reqwest` for HTTP client
- `axum` for local file serving
- `tracing` for logging
- `wiremock` + `assert_cmd` for testing

## Release Process

```bash
make gh-release V=x.y.z   # Updates Cargo.toml, commits, tags, pushes
```

This triggers the release workflow which builds binaries for Linux, macOS (Intel/ARM), and Windows.
