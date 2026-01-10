# AudioShake API Integration Guide

**Version:** 1.0.0
**Date:** January 2026

This document provides comprehensive details on integrating with the AudioShake API, including account setup, authentication, endpoint specifications, and response handling.

---

## Table of Contents

1. [Getting Started](#getting-started)
2. [Authentication](#authentication)
3. [Base URL](#base-url)
4. [API Endpoints](#api-endpoints)
5. [Models Reference](#models-reference)
6. [Supported Formats](#supported-formats)
7. [Supported Languages](#supported-languages)
8. [Error Handling](#error-handling)
9. [Rate Limits](#rate-limits)
10. [Webhooks](#webhooks)
11. [CI/CD Integration](#cicd-integration)
12. [Publishing to crates.io](#publishing-to-cratesio)

---

## Getting Started

This section walks you through setting up an AudioShake account and obtaining API access.

### Step 1: Create an Account

1. Visit the [AudioShake Website](https://www.audioshake.ai/)
2. Click **Sign Up** or **Get Started**
3. Complete the registration form with your email and password
4. Verify your email address via the confirmation link

### Step 2: Access the Dashboard

1. Log in to the [AudioShake Dashboard](https://dashboard.audioshake.ai/)
2. This is your central hub for:
   - Managing API keys
   - Viewing usage statistics
   - Monitoring credit balance
   - Accessing billing information

### Step 3: Purchase Credits

AudioShake uses a credit-based pricing model. Credits are consumed when processing audio.

**Pricing:**

- **Minimum purchase**: 200 credits (~$35 USD)
- **Credit cost**: Varies by model (typically 1.0-1.5 credits per minute of audio)

**How to purchase:**

1. Log in to the [Dashboard](https://dashboard.audioshake.ai/)
2. Navigate to **Billing** or **Credits**
3. Select a credit package
4. Complete payment via credit card or other supported methods

**Credit consumption by model:**

| Model Type                     | Credits per Minute |
| ------------------------------ | ------------------ |
| Stem separation (standard)     | 1.0                |
| Stem separation (high-quality) | 1.5                |
| Dialogue/Effects               | 1.5                |
| Music Detection                | 0.5                |
| Transcription                  | 1.0                |
| Alignment                      | 1.0                |
| Transcription + Alignment      | 1.5                |

### Step 4: Generate an API Key

1. In the Dashboard, go to **Settings** → **API Keys**
2. Click **Generate New Key**
3. Give your key a descriptive name (e.g., "audioshake-cli-production")
4. Copy the key immediately - it won't be shown again
5. Store the key securely

> **Important**: API keys start with `ashke_` prefix (e.g., `ashke_abc123...`)

### Step 5: Configure the CLI

Set your API key using one of these methods:

**Option A: Environment Variable (Recommended)**

```bash
export AUDIOSHAKE_API_KEY=ashke_your_api_key_here
```

**Option B: .env File**

```bash
echo "AUDIOSHAKE_API_KEY=ashke_your_api_key_here" > .env
```

**Option C: Config File**

```bash
audioshake config init
audioshake config set api-key ashke_your_api_key_here
```

### Step 6: Validate Setup

```bash
audioshake config validate
```

You should see:

```text
Validating configuration...
  [OK] API key format valid
  [OK] API connection successful
  [OK] Account active
```

### Useful Links

- **Website**: [https://www.audioshake.ai/](https://www.audioshake.ai/)
- **Dashboard**: [https://dashboard.audioshake.ai/](https://dashboard.audioshake.ai/)
- **Developer Portal**: [https://developer.audioshake.ai/](https://developer.audioshake.ai/)
- **Support Email**: support@audioshake.ai

---

## Authentication

The AudioShake API uses API key authentication via the `x-api-key` header.

### Obtaining an API Key

1. Log in to the [AudioShake Dashboard](https://dashboard.audioshake.ai/)
2. Navigate to **Settings** → **API Keys**
3. Click **Generate New Key**
4. Copy and store the key securely (it cannot be retrieved after creation)

### Using the API Key

Include the key in all requests:

```bash
curl -X POST https://api.audioshake.ai/tasks \
  -H "x-api-key: ashke_your_api_key_here" \
  -H "Content-Type: application/json" \
  -d '{"url": "https://example.com/audio.mp3", "targets": [...]}'
```

### Security Best Practices

- Never commit API keys to version control
- Use environment variables or secrets management
- Rotate keys periodically
- Revoke compromised keys immediately via the dashboard

> **Source**: [AudioShake Getting Started](https://developer.audioshake.ai/tasksGettingStarted)

---

## Base URL

All API requests should be made to:

```text
https://api.audioshake.ai
```

---

## API Endpoints

### POST /tasks - Create a Task

Creates a new processing task with up to 10 target operations.

**Request:**

```json
{
  "url": "https://example.com/audio.mp3",
  "callbackUrl": "https://your-server.com/webhook",
  "targets": [
    {
      "model": "vocals",
      "formats": ["wav", "mp3"],
      "variant": "high_quality"
    },
    {
      "model": "instrumental",
      "formats": ["wav"]
    }
  ]
}
```

**Parameters:**

| Field         | Type   | Required | Description                                       |
| ------------- | ------ | -------- | ------------------------------------------------- |
| `url`         | string | Yes      | Publicly accessible HTTPS URL to audio/video file |
| `callbackUrl` | string | No       | Webhook URL for status notifications              |
| `targets`     | array  | Yes      | Array of 1-10 target operations                   |

**Target Object:**

| Field      | Type    | Required | Description                                                  |
| ---------- | ------- | -------- | ------------------------------------------------------------ |
| `model`    | string  | Yes      | Model name (see [Models Reference](#models-reference))       |
| `formats`  | array   | Yes      | Output formats (see [Supported Formats](#supported-formats)) |
| `variant`  | string  | No       | Model variant (e.g., `high_quality`)                         |
| `language` | string  | No       | ISO 639-1 language code for transcription                    |
| `residual` | boolean | No       | Include residual stem                                        |

**Response:**

```json
{
  "id": "task_abc123def456",
  "status": "pending",
  "createdAt": "2026-01-10T10:30:00Z",
  "targets": [
    {
      "model": "vocals",
      "status": "pending"
    },
    {
      "model": "instrumental",
      "status": "pending"
    }
  ]
}
```

> **Source**: [AudioShake Tasks API](https://developer.audioshake.ai/tasks)

---

### GET /tasks - List Tasks

Returns paginated task history.

**Query Parameters:**

| Parameter | Type    | Default | Description                         |
| --------- | ------- | ------- | ----------------------------------- |
| `skip`    | integer | 0       | Number of tasks to skip             |
| `take`    | integer | 20      | Number of tasks to return (max 100) |
| `status`  | string  | -       | Filter by status                    |

**Response:**

```json
{
  "tasks": [
    {
      "id": "task_abc123",
      "status": "completed",
      "createdAt": "2026-01-10T10:30:00Z",
      "completedAt": "2026-01-10T10:32:15Z",
      "targets": [...]
    }
  ],
  "total": 150,
  "offset": 0,
  "limit": 20
}
```

---

### GET /tasks/{id} - Get Task Status

Retrieves detailed status of a specific task.

**Response (Completed):**

```json
{
  "id": "task_abc123",
  "status": "completed",
  "createdAt": "2026-01-10T10:30:00Z",
  "completedAt": "2026-01-10T10:32:15Z",
  "cost": {
    "credits": 2.5,
    "durationMinutes": 2.5
  },
  "targets": [
    {
      "model": "vocals",
      "status": "completed",
      "output": {
        "links": [
          {
            "format": "wav",
            "link": "https://cdn.audioshake.ai/outputs/vocals.wav",
            "size": 12500000
          }
        ]
      }
    }
  ]
}
```

**Response (Failed):**

```json
{
  "id": "task_abc123",
  "status": "failed",
  "createdAt": "2026-01-10T10:30:00Z",
  "error": {
    "code": "INVALID_FORMAT",
    "message": "Unsupported audio format"
  }
}
```

---

### GET /tasks/statistics - Get Statistics

Returns aggregated usage statistics.

**Query Parameters:**

| Parameter | Type   | Description                               |
| --------- | ------ | ----------------------------------------- |
| `name`    | string | Statistic type: `usage` or `distribution` |

**Response:**

```json
{
  "totalTasks": 150,
  "completedTasks": 145,
  "failedTasks": 5,
  "creditsUsed": 1234.5,
  "totalDurationMinutes": 823
}
```

---

## Models Reference

### Stem Separation Models

| Model          | Description                 | Credits/Min | Max Length |
| -------------- | --------------------------- | ----------- | ---------- |
| `vocals`       | Isolated vocal track        | 1.0         | 3 hours    |
| `instrumental` | Instrumental (no vocals)    | 1.0         | 3 hours    |
| `drums`        | Isolated drum track         | 1.0         | 3 hours    |
| `bass`         | Isolated bass track         | 1.0         | 3 hours    |
| `guitar`       | Isolated guitar track       | 1.0         | 3 hours    |
| `piano`        | Isolated piano track        | 1.0         | 3 hours    |
| `strings`      | Isolated string instruments | 1.0         | 3 hours    |
| `wind`         | Isolated wind instruments   | 1.0         | 3 hours    |
| `other`        | Residual audio              | 1.0         | 3 hours    |

**High-Quality Variants:**

- `vocals` with `variant: "high_quality"` - 1.5 credits/min
- `instrumental` with `variant: "high_quality"` - 1.5 credits/min

### Dialogue/Music/Effects Models

| Model             | Description                 | Credits/Min | Max Length |
| ----------------- | --------------------------- | ----------- | ---------- |
| `dialogue`        | Isolate speech from audio   | 1.5         | 3 hours    |
| `effects`         | Sound effects and ambience  | 1.5         | 3 hours    |
| `music_detection` | Detect music sections       | 0.5         | 3 hours    |
| `background`      | Background audio extraction | 1.5         | 3 hours    |
| `music_removal`   | Remove music, keep dialogue | 1.0         | 1 hour     |

### Multi-Voice Models

| Model         | Description                | Credits/Min |
| ------------- | -------------------------- | ----------- |
| `two_speaker` | Separate two speakers      | N/A         |
| `n_speaker`   | Separate multiple speakers | N/A         |

_Note: Multi-voice models may not be available via /tasks route._

### Transcription Models

| Model           | Description            | Credits/Min |
| --------------- | ---------------------- | ----------- |
| `transcription` | Convert speech to text | 1.0         |
| `alignment`     | Word-level timestamps  | 1.0         |

_Combined transcription + alignment: 1.5 credits/min_

> **Source**: [AudioShake Models](https://developer.audioshake.ai/models)

---

## Supported Formats

### Input Formats

| Format | Extension  | Description                                    |
| ------ | ---------- | ---------------------------------------------- |
| WAV    | .wav       | Waveform Audio (recommended: 16-bit or 24-bit) |
| MP3    | .mp3       | MPEG Audio Layer III                           |
| FLAC   | .flac      | Free Lossless Audio Codec                      |
| AAC    | .aac, .m4a | Advanced Audio Coding                          |
| AIFF   | .aiff      | Audio Interchange File Format                  |
| PCM    | -          | Pulse Code Modulation                          |

### Video Containers

| Format | Extension | Notes                           |
| ------ | --------- | ------------------------------- |
| MP4    | .mp4      | Audio track must use AAC or PCM |
| MOV    | .mov      | QuickTime container             |

### Output Formats

**Audio:**

- `wav` - Uncompressed WAV
- `mp3` - MP3 (compressed)
- `flac` - FLAC (lossless)
- `aiff` - AIFF

**Text (for transcription):**

- `txt` - Plain text
- `json` - JSON with timestamps

### Limitations

- Maximum file length: 3 hours (1 hour for some models)
- Maximum resolution: 192kHz
- DRM-encrypted files are not supported
- Multi-channel surround (5.1) is not supported

> **Source**: [AudioShake Supported Formats](https://developer.audioshake.ai/supported-format)

---

## Supported Languages

The transcription and alignment models support **73 languages**. Include the `language` field in your target to improve accuracy.

### Common Language Codes

| Code | Language   | Code | Language   |
| ---- | ---------- | ---- | ---------- |
| en   | English    | es   | Spanish    |
| fr   | French     | de   | German     |
| it   | Italian    | pt   | Portuguese |
| nl   | Dutch      | pl   | Polish     |
| ru   | Russian    | ja   | Japanese   |
| zh   | Chinese    | ko   | Korean     |
| ar   | Arabic     | hi   | Hindi      |
| vi   | Vietnamese | th   | Thai       |
| tr   | Turkish    | sv   | Swedish    |
| da   | Danish     | no   | Norwegian  |
| fi   | Finnish    | he   | Hebrew     |
| id   | Indonesian | ms   | Malay      |
| cs   | Czech      | el   | Greek      |
| hu   | Hungarian  | ro   | Romanian   |
| sk   | Slovak     | uk   | Ukrainian  |
| bg   | Bulgarian  | hr   | Croatian   |
| sl   | Slovenian  | lt   | Lithuanian |
| lv   | Latvian    | et   | Estonian   |
| ta   | Tamil      | te   | Telugu     |
| ml   | Malayalam  | kn   | Kannada    |
| mr   | Marathi    | gu   | Gujarati   |
| bn   | Bengali    | pa   | Punjabi    |
| ur   | Urdu       | fa   | Persian    |
| sw   | Swahili    | af   | Afrikaans  |
| ca   | Catalan    | gl   | Galician   |
| eu   | Basque     | is   | Icelandic  |

> **Source**: [AudioShake Supported Languages](https://developer.audioshake.ai/supported-languages)

---

## Error Handling

### HTTP Status Codes

| Status | Description                                            |
| ------ | ------------------------------------------------------ |
| 200    | Success                                                |
| 400    | Bad Request (invalid parameters, insufficient credits) |
| 401    | Unauthorized (invalid API key)                         |
| 404    | Not Found (task not found)                             |
| 429    | Too Many Requests (rate limit exceeded)                |
| 500    | Internal Server Error                                  |

### Error Response Format

```json
{
  "error": {
    "code": "ERROR_CODE",
    "message": "Human-readable error message"
  }
}
```

### Common Error Codes

| Code                   | Description                   | Resolution                      |
| ---------------------- | ----------------------------- | ------------------------------- |
| `INVALID_API_KEY`      | API key is invalid or expired | Check key in dashboard          |
| `INSUFFICIENT_CREDITS` | Not enough credits            | Add credits to account          |
| `INVALID_URL`          | Cannot access media URL       | Ensure URL is public HTTPS      |
| `UNSUPPORTED_FORMAT`   | File format not supported     | Convert to supported format     |
| `FILE_TOO_LONG`        | File exceeds maximum length   | Split file or use shorter audio |
| `RATE_LIMITED`         | Too many requests             | Wait and retry                  |

---

## Rate Limits

The API enforces rate limits to ensure fair usage:

- **Requests per minute**: Varies by plan
- **Concurrent tasks**: Varies by plan
- **Daily credits**: Based on account balance

When rate limited, the API returns HTTP 429 with a `Retry-After` header.

### Handling Rate Limits

```rust
async fn with_retry<T, F, Fut>(mut f: F, max_retries: u32) -> Result<T>
where
    F: FnMut() -> Fut,
    Fut: Future<Output = Result<T>>,
{
    let mut attempt = 0;
    loop {
        match f().await {
            Ok(result) => return Ok(result),
            Err(e) if e.is_rate_limited() && attempt < max_retries => {
                attempt += 1;
                let delay = Duration::from_secs(2u64.pow(attempt));
                tokio::time::sleep(delay).await;
            }
            Err(e) => return Err(e),
        }
    }
}
```

---

## Webhooks

Configure a callback URL to receive notifications when task status changes.

### Webhook Payload

```json
{
  "event": "task.completed",
  "taskId": "task_abc123",
  "status": "completed",
  "timestamp": "2026-01-10T10:32:15Z"
}
```

### Event Types

- `task.pending` - Task created
- `task.processing` - Task started processing
- `task.completed` - Task completed successfully
- `task.failed` - Task failed

### Webhook Security

- Verify the webhook signature (if provided)
- Use HTTPS for your callback URL
- Respond with 200 status within 30 seconds
- Implement idempotency for duplicate deliveries

---

## CI/CD Integration

### GitHub Actions Workflows

The project includes comprehensive CI/CD automation based on proven patterns.

#### CI Workflow (.github/workflows/ci.yml)

The CI workflow runs on all pushes and pull requests:

**Jobs:**

1. **lint** - Format check and Clippy linting
2. **security** - Cargo audit for vulnerabilities
3. **test** - Unit and integration tests
4. **build** - Release binary compilation

**Key Features:**

- sccache for faster Rust builds
- cargo-nextest for parallel test execution
- Dependency caching
- Test result artifacts

#### Release Workflow (.github/workflows/release.yml)

The release workflow triggers on version tags (v\*):

**Jobs:**

1. **create-release** - Create GitHub release with changelog
2. **build-binaries** - Cross-platform binary builds
3. **publish-crate** - Publish to crates.io

**Supported Platforms:**

- Linux x86_64
- macOS x86_64 (Intel)
- macOS aarch64 (Apple Silicon)
- Windows x86_64

### Environment Variables for CI

| Variable              | Description                          |
| --------------------- | ------------------------------------ |
| `CARGO_TERM_COLOR`    | Enable colored output                |
| `RUSTFLAGS`           | Compiler flags (e.g., `-D warnings`) |
| `SCCACHE_GHA_ENABLED` | Enable GitHub Actions sccache        |

### Required Secrets

| Secret                 | Description                        |
| ---------------------- | ---------------------------------- |
| `CARGO_REGISTRY_TOKEN` | crates.io API token for publishing |

---

## Publishing to crates.io

### Prerequisites

1. Create account at [crates.io](https://crates.io)
2. Generate API token: Settings → API Tokens
3. Add token as GitHub secret: `CARGO_REGISTRY_TOKEN`

### Cargo.toml Requirements

```toml
[package]
name = "audioshake-cli"
version = "1.0.0"
edition = "2024"
authors = ["Your Name <email@example.com>"]
description = "Command-line interface for the AudioShake audio processing API"
license = "MIT"
repository = "https://github.com/your-org/audioshake-cli"
homepage = "https://github.com/your-org/audioshake-cli"
documentation = "https://docs.rs/audioshake-cli"
readme = "README.md"
keywords = ["audio", "stems", "separation", "transcription", "cli"]
categories = ["command-line-utilities", "multimedia::audio"]

# Required for crates.io
include = [
    "src/**/*",
    "Cargo.toml",
    "README.md",
    "LICENSE",
]
```

### Publishing Steps

#### Manual Publishing

```bash
# 1. Ensure you're logged in
cargo login

# 2. Run checks
cargo fmt --check
cargo clippy -- -D warnings
cargo test

# 3. Verify package
cargo package --list

# 4. Dry run
cargo publish --dry-run

# 5. Publish
cargo publish
```

#### Automated Publishing (via Release Workflow)

1. Update version in Cargo.toml
2. Create and push a version tag:

```bash
git tag v1.0.0
git push origin v1.0.0
```

3. The release workflow will:
   - Create a GitHub release
   - Build binaries for all platforms
   - Publish to crates.io

### Versioning Guidelines

Follow [Semantic Versioning](https://semver.org/):

- **MAJOR** (1.0.0 → 2.0.0): Breaking API changes
- **MINOR** (1.0.0 → 1.1.0): New features, backward compatible
- **PATCH** (1.0.0 → 1.0.1): Bug fixes, backward compatible

### Pre-release Versions

For testing before stable release:

```toml
version = "1.0.0-beta.1"
```

Pre-releases won't be installed by default with `cargo install`.

---

## References

### Official Documentation

- [AudioShake Developer Portal](https://developer.audioshake.ai/)
- [AudioShake Tasks API](https://developer.audioshake.ai/tasks)
- [AudioShake Getting Started](https://developer.audioshake.ai/tasksGettingStarted)
- [AudioShake Models](https://developer.audioshake.ai/models)
- [AudioShake Supported Formats](https://developer.audioshake.ai/supported-format)
- [AudioShake Supported Languages](https://developer.audioshake.ai/supported-languages)

### Rust Resources

- [The Rust Programming Language](https://doc.rust-lang.org/book/)
- [Clap Documentation](https://docs.rs/clap/latest/clap/)
- [Tokio Tutorial](https://tokio.rs/tokio/tutorial)
- [Reqwest Documentation](https://docs.rs/reqwest/latest/reqwest/)
- [crates.io Publishing Guide](https://doc.rust-lang.org/cargo/reference/publishing.html)

### CI/CD Resources

- [GitHub Actions Documentation](https://docs.github.com/en/actions)
- [Rust GitHub Actions](https://github.com/actions-rs)
- [sccache](https://github.com/mozilla/sccache)
- [cargo-nextest](https://nexte.st/)
