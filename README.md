# AudioShake CLI

[![CI](https://github.com/pacphi/audioshake-cli/actions/workflows/ci.yml/badge.svg)](https://github.com/pacphi/audioshake-cli/actions/workflows/ci.yml)
[![codecov](https://codecov.io/gh/pacphi/audioshake-cli/branch/main/graph/badge.svg)](https://codecov.io/gh/pacphi/audioshake-cli)
[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](LICENSE)

A high-performance command-line interface for the [AudioShake API](https://developer.audioshake.ai/), built in Rust.

## Features

- **Stem Separation**: Extract vocals, drums, bass, guitar, piano, and other instruments
- **Transcription**: Convert speech to text with word-level timestamps
- **Dialogue Extraction**: Isolate speech from audio/video for post-production
- **Local File Support**: Process local files via Cloudflare R2 cloud storage
- **Async Processing**: Submit tasks and monitor progress or use webhooks
- **Cross-Platform**: Runs on macOS, Linux, and Windows

## Installation

### From Cargo

```bash
cargo install audioshake-cli
```

### From Binary Releases

Download the latest release for your platform from the GitHub Releases page (once published).

## Quick Start

1. Get an API key from the [AudioShake Dashboard](https://dashboard.audioshake.ai)

2. Set your API key:

```bash
export AUDIOSHAKE_API_KEY=ashke_your_api_key_here
# Or create a .env file
echo "AUDIOSHAKE_API_KEY=ashke_your_api_key_here" > .env
```

3. Process an audio file:

```bash
# Extract vocals from a song
audioshake process song.mp3 -m vocals --wait

# High-quality stem separation
audioshake process track.wav -m vocals -m instrumental --hq --wait -o ./stems

# Transcribe audio
audioshake process podcast.mp3 -m transcription -f json --wait
```

## Usage Examples

### Stem Separation

```bash
# Basic vocal extraction
audioshake process song.mp3 -m vocals --wait

# Full band separation
audioshake process song.mp3 -m vocals -m drums -m bass -m guitar -m piano --wait

# High-quality variant (1.5 credits/min)
audioshake process track.wav -m vocals -m instrumental --hq --wait
```

### Transcription

```bash
# Text transcription
audioshake process interview.mp3 -m transcription -f txt --wait

# With word-level timestamps
audioshake process podcast.mp3 -m transcription -m alignment -f json --wait

# Spanish transcription
audioshake process audio.mp3 -m transcription --language es --wait
```

### Video Processing

```bash
# Extract dialogue from video
audioshake process video.mp4 -m dialogue -m effects -f wav --wait
```

### Task Management

```bash
# Submit task without waiting
audioshake process song.mp3 -m vocals
# Output: Task created: abc123-def456

# Check status
audioshake task status abc123-def456

# Download results when complete
audioshake task download abc123-def456 -o ./output
```

## Commands

| Command                      | Description                       |
| ---------------------------- | --------------------------------- |
| `audioshake process`         | Submit audio/video for processing |
| `audioshake task list`       | List tasks                        |
| `audioshake task status`     | Get task status                   |
| `audioshake task download`   | Download task outputs             |
| `audioshake task wait`       | Wait for task completion          |
| `audioshake task statistics` | Show usage statistics             |
| `audioshake models`          | List available models             |
| `audioshake config`          | Manage configuration              |
| `audioshake completion`      | Generate shell completions        |

## Configuration

Configuration is loaded from (in order of priority):

1. `--api-key` command line flag
2. `AUDIOSHAKE_API_KEY` environment variable
3. `.env` file in current directory
4. `~/.config/audioshake/config.toml`

### Config File Example

```toml
api_key = "ashke_your_api_key_here"
base_url = "https://api.audioshake.ai"
default_output_dir = "./output"
default_format = "wav"
default_poll_interval = 5
default_timeout = 3600

# Cloud storage for local file uploads (required for local file processing)
[storage]
provider = "r2"
bucket = "my-audioshake-uploads"
public_url_base = "https://my-audioshake-uploads.r2.dev"

[storage.r2]
account_id = "your_cloudflare_account_id"
access_key_id = "your_r2_access_key"
# secret_access_key set via AUDIOSHAKE_R2_SECRET_ACCESS_KEY env var
```

### Cloud Storage Setup (for Local Files)

The AudioShake Tasks API requires publicly accessible URLs. To process local files, you need to configure cloud storage (Cloudflare R2 recommended for zero egress fees):

1. **Create a Cloudflare account** at [dash.cloudflare.com](https://dash.cloudflare.com)

2. **Create an R2 bucket**:
   - Go to R2 Object Storage
   - Create a bucket (e.g., `audioshake-uploads`)
   - Enable public access in Settings (r2.dev subdomain)

3. **Create an R2 API token**:
   - Go to R2 → Manage API Tokens
   - Create a token with "Object Read & Write" permissions

4. **Configure the CLI**:

```bash
# Via environment variables
export AUDIOSHAKE_STORAGE_PROVIDER=r2
export AUDIOSHAKE_STORAGE_BUCKET=my-audioshake-uploads
export AUDIOSHAKE_STORAGE_PUBLIC_URL_BASE=https://my-audioshake-uploads.r2.dev
export AUDIOSHAKE_R2_ACCOUNT_ID=your_account_id
export AUDIOSHAKE_R2_ACCESS_KEY_ID=your_access_key
export AUDIOSHAKE_R2_SECRET_ACCESS_KEY=your_secret_key

# Or add to .env file
```

See [ADR-002](docs/adr/002-cloud-storage-upload-strategy.md) for technical details.

## Available Models

### Stem Separation

- `vocals` - Isolated vocal track
- `instrumental` - Instrumental (no vocals)
- `drums` - Drum track
- `bass` - Bass track
- `guitar` - Guitar track
- `piano` - Piano track
- `strings` - String instruments
- `wind` - Wind instruments

### Dialogue/Music/Effects

- `dialogue` - Isolated speech
- `effects` - Sound effects and ambience
- `music_detection` - Detect music sections

### Transcription

- `transcription` - Speech to text
- `alignment` - Word-level timestamps

### Multi-Speaker Separation (Enterprise)

- `multi_voice` - Separate individual speakers into distinct tracks

> **Note:** The `multi_voice` model requires enterprise account enablement. Standard developer accounts will receive an "Insufficient credits" error even with available credits. Contact [info@audioshake.ai](mailto:info@audioshake.ai) to request access.
>
> See: [AudioShake Multi-Speaker Separation](https://www.audioshake.ai/products/multi-speaker-separation)

```bash
# Requires enterprise access
audioshake process podcast.mp3 -m multi-voice --variant n_speaker --wait
```

### Sample interaction with Claude Code

```text

 ▐▛███▜▌   Claude Code v2.1.3
▝▜█████▛▘  Opus 4.5 · Claude Max
  ▘▘ ▝▝    ~/Documents/development/ai/audioshake-cli

❯ I need to isolate the voices from music inside this video:
  /Users/cphillipson/Movies/build-with-me/build-with-me-episode7-2026-01-10.mp4. Is there a CLI
  command available that will do that for me?

⏺ Based on the audioshake-cli project you have open, yes! This CLI can separate vocals from music. Let me check the exact command syntax for you.

⏺ Bash(cargo run -- process --help 2>/dev/null || cargo run -- --help 2>/dev/null) timeout: 1m 0s
  ⎿  Command-line interface for the AudioShake audio processing API

     Usage: audioshake [OPTIONS] <COMMAND>
     … +24 lines (ctrl+o to expand)

⏺ To isolate voices (vocals) from your video, run:

  audioshake process "/Users/cphillipson/Movies/build-with-me/build-with-me-episode7-2026-01-10.mp4" --model vocals --wait --output-dir ./output

  This will:
  1. Submit your video file for processing
  2. Use the vocals model to separate voices from music
  3. Wait for completion and download the result to ./output

  Note: You'll need an AUDIOSHAKE_API_KEY environment variable set (or pass --api-key).

  If you also want the instrumental track (music without vocals), add another model:
  audioshake process "/Users/cphillipson/Movies/build-with-me/build-with-me-episode7-2026-01-10.mp4" --model vocals --model instrumental --wait
```

## Documentation

- [CLI Reference](docs/CLI_REFERENCE.md) - Complete command documentation
- [API Integration](docs/API_INTEGRATION.md) - API details and examples
- [Technical Spec](docs/TECHNICAL_SPEC.md) - Architecture and implementation
- [PRD](docs/PRD.md) - Product requirements

## Development

```bash
# Clone repository
git clone https://github.com/pacphi/audioshake-cli.git
cd audioshake-cli

# Install dependencies
make install

# Build
make build

# Run tests
make test

# Run linter
make lint
```

## License

MIT License - see [LICENSE](LICENSE) for details.

## Support

- [AudioShake Developer Portal](https://developer.audioshake.ai/)
