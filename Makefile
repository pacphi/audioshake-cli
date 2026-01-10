# AudioShake CLI Makefile
# Unified interface for build, test, and development operations

.PHONY: help
.PHONY: install build build-release clean
.PHONY: dev run
.PHONY: test test-unit test-integration test-coverage
.PHONY: lint lint-fix format format-check
.PHONY: lint-md lint-md-fix format-md format-md-check lint-yaml check-links
.PHONY: audit license-check
.PHONY: outdated upgrade upgrade-latest
.PHONY: doc doc-open
.PHONY: gh-ci gh-release gh-watch gh-runs gh-status
.PHONY: publish publish-dry-run

# Default target
.DEFAULT_GOAL := help

# =============================================================================
# Help
# =============================================================================

help:
	@echo "AudioShake CLI Makefile - Unified build interface"
	@echo ""
	@echo "Setup & Dependencies:"
	@echo "  install          - Install development dependencies"
	@echo ""
	@echo "Build:"
	@echo "  build            - Build project (debug)"
	@echo "  build-release    - Build project (release)"
	@echo "  clean            - Clean build artifacts"
	@echo ""
	@echo "Development:"
	@echo "  dev              - Run with debug logging"
	@echo "  run ARGS='...'   - Run with custom arguments"
	@echo ""
	@echo "Testing:"
	@echo "  test             - Run all tests"
	@echo "  test-unit        - Run unit tests only"
	@echo "  test-integration - Run integration tests (requires mock server)"
	@echo "  test-coverage    - Run tests with coverage report"
	@echo ""
	@echo "Code Quality:"
	@echo "  lint             - Run clippy linter"
	@echo "  lint-fix         - Auto-fix lint issues"
	@echo "  format           - Format code with rustfmt"
	@echo "  format-check     - Check formatting without changes"
	@echo ""
	@echo "Markdown & YAML:"
	@echo "  lint-md          - Lint markdown files"
	@echo "  lint-md-fix      - Auto-fix markdown lint issues"
	@echo "  format-md        - Format markdown files with prettier"
	@echo "  format-md-check  - Check markdown formatting"
	@echo "  lint-yaml        - Lint YAML files (requires yamllint)"
	@echo "  check-links      - Check links in markdown files (requires lychee)"
	@echo ""
	@echo "Security & Dependencies:"
	@echo "  audit            - Run security audit"
	@echo "  license-check    - Check license compliance"
	@echo "  outdated         - List outdated dependencies"
	@echo "  upgrade          - Upgrade dependencies"
	@echo ""
	@echo "Documentation:"
	@echo "  doc              - Generate documentation"
	@echo "  doc-open         - Generate and open documentation"
	@echo ""
	@echo "GitHub Actions (requires gh CLI):"
	@echo "  gh-ci            - Trigger CI workflow on current branch"
	@echo "  gh-release V=x.y.z - Create and push a release tag"
	@echo "  gh-watch         - Watch the latest workflow run"
	@echo "  gh-runs          - List recent workflow runs"
	@echo "  gh-status        - View CI workflow status"
	@echo ""
	@echo "Publishing:"
	@echo "  publish-dry-run  - Verify crate can be published"
	@echo "  publish          - Publish to crates.io"

# =============================================================================
# Setup & Dependencies
# =============================================================================

install:
	@echo "==> Checking Rust toolchain..."
	rustup show
	@echo ""
	@echo "==> Installing development tools..."
	@command -v cargo-nextest >/dev/null 2>&1 || cargo install cargo-nextest --locked
	@command -v cargo-audit >/dev/null 2>&1 || cargo install cargo-audit --locked
	@command -v cargo-llvm-cov >/dev/null 2>&1 || cargo install cargo-llvm-cov --locked
	@rustup component add llvm-tools-preview 2>/dev/null || true
	@echo ""
	@echo "==> Development tools installed!"

# =============================================================================
# Build
# =============================================================================

build:
	@echo "==> Building (debug)..."
	cargo build

build-release:
	@echo "==> Building (release)..."
	cargo build --release

clean:
	@echo "==> Cleaning..."
	cargo clean

# =============================================================================
# Development
# =============================================================================

dev:
	@echo "==> Running with debug logging..."
	RUST_LOG=debug cargo run -- --help

# Run with custom arguments
# Usage: make run ARGS='process song.mp3 -m vocals'
run:
	@echo "==> Running audioshake..."
	cargo run -- $(ARGS)

# =============================================================================
# Testing
# =============================================================================

test: test-unit

test-unit:
	@echo "==> Running unit tests..."
	@command -v cargo-nextest >/dev/null 2>&1 || cargo install cargo-nextest --locked
	cargo nextest run --all-features

test-integration:
	@echo "==> Running integration tests..."
	@command -v cargo-nextest >/dev/null 2>&1 || cargo install cargo-nextest --locked
	cargo nextest run --all-features --test '*'

test-coverage:
	@echo "==> Running tests with coverage..."
	@command -v cargo-llvm-cov >/dev/null 2>&1 || cargo install cargo-llvm-cov --locked
	@rustup component add llvm-tools-preview 2>/dev/null || true
	@mkdir -p coverage
	cargo llvm-cov \
		--all-features \
		--workspace \
		--html \
		--output-dir coverage
	cargo llvm-cov \
		--all-features \
		--workspace \
		--lcov \
		--output-path coverage/lcov.info
	@echo ""
	@echo "==> Coverage report: coverage/html/index.html"

# =============================================================================
# Code Quality
# =============================================================================

lint:
	@echo "==> Running clippy..."
	cargo clippy --all-targets --all-features -- -D warnings -D clippy::all -D clippy::pedantic

lint-fix:
	@echo "==> Auto-fixing lint issues..."
	cargo clippy --all-targets --all-features --fix --allow-dirty --allow-staged

format:
	@echo "==> Formatting code..."
	cargo fmt --all

format-check:
	@echo "==> Checking formatting..."
	cargo fmt --all -- --check

# =============================================================================
# Markdown & YAML
# =============================================================================

# Check if pnpm is installed and dependencies are available
_check-pnpm:
	@command -v pnpm >/dev/null 2>&1 || { echo "Error: pnpm is not installed. Install from https://pnpm.io/"; exit 1; }
	@test -d node_modules || { echo "Installing pnpm dependencies..."; pnpm install; }

lint-md: _check-pnpm
	@echo "==> Linting markdown files..."
	pnpm lint:md

lint-md-fix: _check-pnpm
	@echo "==> Fixing markdown lint issues..."
	pnpm lint:md:fix

format-md: _check-pnpm
	@echo "==> Formatting markdown files..."
	pnpm format

format-md-check: _check-pnpm
	@echo "==> Checking markdown formatting..."
	pnpm format:check

lint-yaml:
	@command -v yamllint >/dev/null 2>&1 || { echo "Error: yamllint is not installed. Install with: pip install yamllint"; exit 1; }
	@echo "==> Linting YAML files..."
	yamllint .

check-links:
	@command -v lychee >/dev/null 2>&1 || { echo "Error: lychee is not installed. Install with: brew install lychee (macOS) or cargo install lychee"; exit 1; }
	@echo "==> Checking links in markdown files..."
	lychee --verbose --no-progress --exclude-path node_modules --exclude-path .git --exclude-path target --exclude-path undefined --exclude 'https://api\.audioshake\.ai' '**/*.md'

# =============================================================================
# Security & Dependencies
# =============================================================================

audit:
	@echo "==> Running security audit..."
	@command -v cargo-audit >/dev/null 2>&1 || cargo install cargo-audit --locked
	cargo audit

license-check:
	@echo "==> Checking license compliance..."
	@command -v cargo-deny >/dev/null 2>&1 || cargo install cargo-deny --locked
	cargo deny check licenses

outdated:
	@echo "==> Checking for outdated dependencies..."
	@command -v cargo-outdated >/dev/null 2>&1 || cargo install cargo-outdated
	cargo outdated -R

upgrade:
	@echo "==> Upgrading dependencies..."
	cargo update
	@echo "Dependencies updated in Cargo.lock"

upgrade-latest:
	@echo "==> Upgrading to latest versions..."
	@command -v cargo-edit >/dev/null 2>&1 || cargo install cargo-edit
	cargo upgrade
	cargo update

# =============================================================================
# Documentation
# =============================================================================

doc:
	@echo "==> Generating documentation..."
	cargo doc --no-deps --all-features

doc-open:
	@echo "==> Generating and opening documentation..."
	cargo doc --no-deps --all-features --open

# =============================================================================
# CI helpers (used by GitHub Actions)
# =============================================================================

ci: format-check lint test build-release
	@echo "==> All CI checks passed!"

# =============================================================================
# GitHub Actions (requires gh CLI)
# =============================================================================

# Check if gh CLI is available
.PHONY: _check-gh
_check-gh:
	@command -v gh >/dev/null 2>&1 || { echo "Error: gh CLI is not installed. Install from https://cli.github.com/"; exit 1; }
	@gh auth status >/dev/null 2>&1 || { echo "Error: gh CLI is not authenticated. Run 'gh auth login'"; exit 1; }

# Trigger CI workflow on current branch
gh-ci: _check-gh
	@echo "==> Triggering CI workflow..."
	@BRANCH=$$(git rev-parse --abbrev-ref HEAD); \
	gh workflow run ci.yml --ref $$BRANCH; \
	echo "CI workflow triggered on branch: $$BRANCH"; \
	echo "Run 'make gh-watch' to monitor progress"

# Create a release tag and trigger release workflow
# Usage: make gh-release V=1.0.0
gh-release: _check-gh
ifndef V
	$(error Version not specified. Usage: make gh-release V=x.y.z)
endif
	@echo "==> Creating release v$(V)..."
	@if git rev-parse "v$(V)" >/dev/null 2>&1; then \
		echo "Error: Tag v$(V) already exists"; \
		exit 1; \
	fi
	@# Update version in Cargo.toml
	@echo "Updating version in Cargo.toml to $(V)..."
	@sed -i.bak 's/^version = .*/version = "$(V)"/' Cargo.toml && rm Cargo.toml.bak
	@git add Cargo.toml
	@git commit -m "chore: bump version to $(V)"
	@git tag -a "v$(V)" -m "Release v$(V)"
	@git push origin HEAD
	@git push origin "v$(V)"
	@echo "Release v$(V) tag pushed. Release workflow will start automatically."
	@echo "Run 'make gh-watch' to monitor progress"

# Watch the latest workflow run
gh-watch: _check-gh
	@echo "==> Watching latest workflow run..."
	@gh run watch

# List recent workflow runs
gh-runs: _check-gh
	@gh run list --limit 10

# View CI workflow status
gh-status: _check-gh
	@gh run list --workflow=ci.yml --limit 5

# =============================================================================
# Publishing
# =============================================================================

publish-dry-run:
	@echo "==> Verifying package..."
	cargo package --list
	cargo publish --dry-run

publish:
	@echo "==> Publishing to crates.io..."
	@echo "Make sure you have CARGO_REGISTRY_TOKEN set or run 'cargo login'"
	cargo publish

# =============================================================================
# Utility targets
# =============================================================================

# Check cloudflared is installed (required for local file processing)
check-cloudflared:
	@command -v cloudflared >/dev/null 2>&1 || { \
		echo "cloudflared is not installed."; \
		echo "Install with:"; \
		echo "  macOS:  brew install cloudflared"; \
		echo "  Linux:  See https://github.com/cloudflare/cloudflared/releases"; \
		exit 1; \
	}
	@echo "cloudflared is installed: $$(cloudflared --version)"

# Verify .env file exists
check-env:
	@test -f .env || { \
		echo ".env file not found. Create one with:"; \
		echo "  echo 'AUDIOSHAKE_API_KEY=your_key_here' > .env"; \
		exit 1; \
	}
	@echo ".env file found"

# Full setup check
check: check-env check-cloudflared
	@echo ""
	@echo "==> All checks passed! You're ready to use AudioShake CLI."
