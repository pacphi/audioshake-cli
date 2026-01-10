# Release Process

This document describes how to create releases for AudioShake CLI using Git and GitHub automation.

## Versioning

AudioShake CLI follows [Semantic Versioning](https://semver.org/):

- **MAJOR** (1.0.0): Breaking changes to CLI interface or configuration
- **MINOR** (0.1.0): New features, backwards compatible
- **PATCH** (0.0.1): Bug fixes, backwards compatible

## Release Workflow

### 1. Prepare the Release

```bash
# Ensure you're on main with latest changes
git checkout main
git pull origin main

# Create a release branch
git checkout -b release/v0.2.0
```

### 2. Update Version Number

Update the version in `Cargo.toml`:

```bash
# Edit Cargo.toml
sed -i '' 's/version = "0.1.0"/version = "0.2.0"/' Cargo.toml
```

Or manually edit the `[package]` section:

```toml
[package]
name = "audioshake-cli"
version = "0.2.0"
```

### 3. Verify the Build

```bash
# Ensure the project builds and tests pass
cargo build --release
cargo test
cargo clippy -- -D warnings
```

### 4. Create Release Commit

```bash
git add Cargo.toml Cargo.lock
git commit -m "chore: prepare release v0.2.0"
git push origin release/v0.2.0
```

### 5. Create Pull Request

Open a PR from `release/v0.2.0` to `main` for final review.

### 6. Tag and Release

After merging to main:

```bash
git checkout main
git pull origin main

# Create annotated tag
git tag -a v0.2.0 -m "Release v0.2.0"

# Push tag
git push origin v0.2.0
```

## GitHub Release Automation

When you push a tag, the GitHub Actions workflow (`.github/workflows/release.yml`) automatically:

1. Creates a GitHub Release with auto-generated changelog
2. Builds release binaries for all supported platforms
3. Uploads binary artifacts to the release
4. Optionally publishes to crates.io (if configured)

### Release Trigger

The release workflow is triggered on version tags:

```yaml
on:
  push:
    tags:
      - 'v*'
```

### Release Artifacts

Each release includes pre-compiled binaries for:

| Platform | Architecture  | Artifact                                   |
| -------- | ------------- | ------------------------------------------ |
| Linux    | x86_64        | `audioshake-{version}-linux-amd64.tar.gz`  |
| macOS    | Intel         | `audioshake-{version}-darwin-amd64.tar.gz` |
| macOS    | Apple Silicon | `audioshake-{version}-darwin-arm64.tar.gz` |
| Windows  | x86_64        | `audioshake-{version}-windows-amd64.zip`   |

### crates.io Publishing

The workflow can optionally publish to [crates.io](https://crates.io). To enable this:

1. Create a crates.io API token at https://crates.io/settings/tokens
2. Add as repository secret: `CARGO_REGISTRY_TOKEN`
3. The workflow will automatically publish after binaries are built

If `CARGO_REGISTRY_TOKEN` is not configured, this step is skipped with a warning.

## Hotfix Process

For urgent fixes to a released version:

```bash
# Create hotfix branch from the release tag
git checkout v0.2.0
git checkout -b hotfix/v0.2.1

# Make fixes
# ... edit files ...

# Update version in Cargo.toml
sed -i '' 's/version = "0.2.0"/version = "0.2.1"/' Cargo.toml

# Commit and push
git add -A
git commit -m "fix: critical bug fix"
git push origin hotfix/v0.2.1

# After PR approval and merge
git checkout main
git pull origin main
git tag -a v0.2.1 -m "Hotfix v0.2.1"
git push origin v0.2.1
```

## Pre-release Versions

For beta or release candidates:

```bash
# Beta release
git tag -a v0.3.0-beta.1 -m "Beta release v0.3.0-beta.1"

# Release candidate
git tag -a v0.3.0-rc.1 -m "Release candidate v0.3.0-rc.1"
```

Pre-release tags (containing `-`) are automatically marked as pre-release in GitHub.

## Installation Methods

After a release, users can install via:

**Via Cargo (if published to crates.io):**

```bash
cargo install audioshake-cli
```

**Via Binary Download:**

```bash
# macOS (Apple Silicon)
curl -LO https://github.com/pacphi/audioshake-cli/releases/latest/download/audioshake-{version}-darwin-arm64.tar.gz
tar -xzf audioshake-{version}-darwin-arm64.tar.gz
sudo mv audioshake /usr/local/bin/

# macOS (Intel)
curl -LO https://github.com/pacphi/audioshake-cli/releases/latest/download/audioshake-{version}-darwin-amd64.tar.gz
tar -xzf audioshake-{version}-darwin-amd64.tar.gz
sudo mv audioshake /usr/local/bin/

# Linux
curl -LO https://github.com/pacphi/audioshake-cli/releases/latest/download/audioshake-{version}-linux-amd64.tar.gz
tar -xzf audioshake-{version}-linux-amd64.tar.gz
sudo mv audioshake /usr/local/bin/
```

## Release Checklist

### Before Releasing

- [ ] All CI checks pass on main
- [ ] Version number updated in `Cargo.toml`
- [ ] `cargo build --release` succeeds
- [ ] `cargo test` passes
- [ ] `cargo clippy -- -D warnings` passes
- [ ] Documentation updated (if applicable)
- [ ] Manual testing completed for key features

### After Releasing

- [ ] Verify GitHub Release created with changelog
- [ ] Verify all platform binaries uploaded
- [ ] Verify crates.io publish (if configured)
- [ ] Download and test a binary from the release
- [ ] Update any dependent documentation or announcements

## Troubleshooting

### Release workflow failed

1. Check the Actions tab for specific error messages
2. Common issues:
   - Build failures on specific platforms
   - Missing secrets for crates.io publishing
   - Network timeouts during artifact upload

### crates.io publish failed

1. Verify `CARGO_REGISTRY_TOKEN` secret is configured
2. Ensure the version in `Cargo.toml` hasn't been published before
3. Check that all metadata in `Cargo.toml` is valid
4. Run `cargo publish --dry-run` locally to verify

### Binary doesn't work on target platform

1. Verify the correct binary was downloaded for the platform
2. Check that the binary has execute permissions: `chmod +x audioshake`
3. On macOS, you may need to allow the binary in System Preferences > Security & Privacy
