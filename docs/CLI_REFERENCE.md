# AudioShake CLI Reference

**Version:** 1.0.0

Complete command-line reference for the AudioShake CLI.

---

## Installation

### From Cargo (Recommended)

```bash
cargo install audioshake-cli
```

### From Binary Release

Download the latest release for your platform from GitHub Releases.

---

## Configuration

### Environment Variable

```bash
export AUDIOSHAKE_API_KEY=ashke_your_api_key_here
```

### .env File

Create `.env` in your working directory:

```text
AUDIOSHAKE_API_KEY=ashke_your_api_key_here
```

### Config File

Create `~/.config/audioshake/config.toml`:

```toml
api_key = "ashke_your_api_key_here"
default_output_dir = "~/Music/stems"
default_format = "wav"
default_poll_interval = 5
default_timeout = 3600
```

### Priority Order

1. `--api-key` flag (highest)
2. `AUDIOSHAKE_API_KEY` environment variable
3. `.env` file in current directory
4. `~/.config/audioshake/config.toml` (lowest)

---

## Global Options

These options can be used with any command:

| Option                  | Short | Description                                       |
| ----------------------- | ----- | ------------------------------------------------- |
| `--api-key <KEY>`       |       | API key (overrides env var)                       |
| `--base-url <URL>`      |       | API base URL (default: https://api.audioshake.ai) |
| `--output-format <FMT>` | `-o`  | Output format: text, json, yaml (default: text)   |
| `--verbose`             | `-v`  | Increase verbosity (-v, -vv, -vvv)                |
| `--quiet`               | `-q`  | Suppress non-essential output                     |
| `--help`                | `-h`  | Print help information                            |
| `--version`             | `-V`  | Print version                                     |

---

## Commands

### audioshake process

Submit audio/video files for processing.

```text
audioshake process [OPTIONS] <INPUT>
```

#### Arguments

| Argument  | Description                  |
| --------- | ---------------------------- |
| `<INPUT>` | Local file path or HTTPS URL |

#### Options

| Option                   | Short | Description                                 |
| ------------------------ | ----- | ------------------------------------------- |
| `--model <MODEL>`        | `-m`  | Model(s) to apply (can be repeated)         |
| `--format <FMT>`         | `-f`  | Output format(s) (can be repeated)          |
| `--hq`                   |       | Use high-quality variant (1.5 credits/min)  |
| `--residual`             |       | Include residual stem                       |
| `--language <LANG>`      |       | Language code for transcription (ISO 639-1) |
| `--callback-url <URL>`   |       | Webhook URL for completion                  |
| `--output-dir <DIR>`     |       | Output directory (default: ./output)        |
| `--wait`                 |       | Wait for completion and download results    |
| `--timeout <SECS>`       |       | Timeout for --wait (default: 3600)          |
| `--poll-interval <SECS>` |       | Polling interval (default: 5)               |
| `--no-download`          |       | With --wait, don't download results         |

#### Available Models

**Stem Separation:**

- `vocals` - Isolated vocal track
- `instrumental` - Instrumental (no vocals)
- `drums` - Drum track
- `bass` - Bass track
- `guitar` - Guitar track
- `piano` - Piano track
- `strings` - String instruments
- `wind` - Wind instruments
- `other` - Residual audio

**Dialogue/Music/Effects:**

- `dialogue` - Isolated speech/vocals
- `effects` - Sound effects and ambience
- `music_detection` - Detect music sections
- `background` - Background audio

**Transcription:**

- `transcription` - Text transcription
- `alignment` - Word-level timestamps

#### Output Formats

**Audio:** `wav`, `mp3`, `flac`, `aiff`
**Text:** `txt`, `json`

#### Examples

```bash
# Basic vocal extraction
audioshake process song.mp3 -m vocals --wait

# High-quality stems with custom output
audioshake process track.wav -m vocals -m instrumental --hq --wait -o ./stems

# Full band separation
audioshake process song.mp3 -m vocals -m drums -m bass -m guitar -m piano --wait

# Transcription with alignment
audioshake process podcast.mp3 -m transcription -m alignment -f json --wait

# Spanish transcription
audioshake process interview.mp3 -m transcription --language es -f txt --wait

# Video dialogue extraction
audioshake process movie.mp4 -m dialogue -m effects -f wav --wait

# Multiple output formats
audioshake process song.mp3 -m vocals -f wav -f mp3 --wait

# Remote URL processing
audioshake process https://example.com/audio.mp3 -m vocals --wait

# Submit without waiting
audioshake process song.mp3 -m vocals
# Output: Task created: abc123-def456

# JSON output for scripting
audioshake -o json process song.mp3 -m vocals --wait
```

---

### audioshake task

Manage and monitor processing tasks.

```text
audioshake task <SUBCOMMAND>
```

#### Subcommands

##### task list

List tasks with optional filtering.

```text
audioshake task list [OPTIONS]
```

| Option              | Description                                    |
| ------------------- | ---------------------------------------------- |
| `--status <STATUS>` | Filter: pending, processing, completed, failed |
| `--limit <N>`       | Maximum tasks to return (default: 20)          |
| `--offset <N>`      | Pagination offset (default: 0)                 |
| `--since <DATE>`    | Tasks after date (ISO 8601)                    |
| `--before <DATE>`   | Tasks before date (ISO 8601)                   |

```bash
# List recent tasks
audioshake task list

# List completed tasks
audioshake task list --status completed --limit 10

# List tasks from this week
audioshake task list --since 2026-01-06

# JSON output for scripting
audioshake -o json task list | jq '.tasks[].id'
```

##### task status

Get detailed status of a specific task.

```text
audioshake task status <TASK_ID>
```

```bash
# Check task status
audioshake task status abc123-def456

# JSON output
audioshake -o json task status abc123-def456
```

**Output Example:**

```text
Task: abc123-def456
Status: completed
Created: 2026-01-10 10:30:00 UTC
Completed: 2026-01-10 10:32:15 UTC
Duration: 2m 15s

Targets:
  [OK] vocals      → vocals.wav (12.5 MB)
  [OK] instrumental → instrumental.wav (14.2 MB)
```

##### task download

Download completed task outputs.

```text
audioshake task download <TASK_ID> [OPTIONS]
```

| Option               | Short | Description                          |
| -------------------- | ----- | ------------------------------------ |
| `--output-dir <DIR>` | `-o`  | Output directory (default: ./output) |
| `--model <MODEL>`    |       | Download specific model only         |
| `--overwrite`        |       | Overwrite existing files             |

```bash
# Download all outputs
audioshake task download abc123-def456

# Download to specific directory
audioshake task download abc123-def456 -o ./stems

# Download only vocals
audioshake task download abc123-def456 --model vocals

# Overwrite existing files
audioshake task download abc123-def456 --overwrite
```

##### task wait

Wait for a task to complete.

```text
audioshake task wait <TASK_ID> [OPTIONS]
```

| Option                   | Description                        |
| ------------------------ | ---------------------------------- |
| `--timeout <SECS>`       | Timeout in seconds (default: 3600) |
| `--poll-interval <SECS>` | Polling interval (default: 5)      |
| `--download`             | Download results when complete     |
| `--output-dir <DIR>`     | Directory for downloads            |

```bash
# Wait for task
audioshake task wait abc123-def456

# Wait and download
audioshake task wait abc123-def456 --download -o ./results

# Custom timeout
audioshake task wait abc123-def456 --timeout 600
```

##### task statistics

Show task statistics and usage.

```text
audioshake task statistics [OPTIONS]
```

| Option            | Description                 |
| ----------------- | --------------------------- |
| `--since <DATE>`  | Stats from date (ISO 8601)  |
| `--before <DATE>` | Stats until date (ISO 8601) |

```bash
# Overall statistics
audioshake task statistics

# This month's usage
audioshake task statistics --since 2026-01-01

# JSON output
audioshake -o json task statistics
```

**Output Example:**

```text
Task Statistics
---------------
Total Tasks:     150
Completed:       145 (96.7%)
Failed:          5 (3.3%)
Credits Used:    1,234.5
Total Duration:  823 minutes
```

---

### audioshake models

List available processing models.

```text
audioshake models [OPTIONS]
```

| Option             | Description                        |
| ------------------ | ---------------------------------- |
| `--category <CAT>` | Filter: stems, dmx, transcription  |
| `--detailed`       | Show detailed info including costs |

```bash
# List all models
audioshake models

# Stem separation models only
audioshake models --category stems

# Detailed view with costs
audioshake models --detailed

# JSON output
audioshake -o json models
```

**Output Example:**

```text
Stem Separation Models
----------------------
vocals        Extract isolated vocal track
instrumental  Generate instrumental version
drums         Isolate drum track
bass          Isolate bass track
guitar        Isolate guitar track
piano         Isolate piano track
strings       Isolate string instruments
wind          Isolate wind instruments
other         Residual audio

Dialogue/Music/Effects Models
-----------------------------
dialogue        Isolate speech from audio
effects         Extract sound effects and ambience
music_detection Detect music sections
background      Extract background audio

Transcription Models
--------------------
transcription   Convert speech to text
alignment       Word-level timestamps
```

---

### audioshake config

Manage CLI configuration.

```text
audioshake config <SUBCOMMAND>
```

#### Subcommands

##### config show

Display current configuration.

```bash
audioshake config show
```

**Output Example:**

```text
Configuration
-------------
API Key:       ashke_****cc4e (from .env)
Base URL:      https://api.audioshake.ai
Output Dir:    ./output
Default Format: wav
Poll Interval: 5s
Timeout:       3600s
```

##### config set

Set a configuration value.

```text
audioshake config set <KEY> <VALUE>
```

Available keys:

- `api-key` - API key
- `base-url` - API base URL
- `default-output-dir` - Default output directory
- `default-format` - Default output format
- `default-poll-interval` - Default polling interval
- `default-timeout` - Default timeout

```bash
audioshake config set default-output-dir ~/Music/stems
audioshake config set default-format mp3
audioshake config set default-poll-interval 10
```

##### config init

Initialize configuration file.

```text
audioshake config init [OPTIONS]
```

| Option    | Description                      |
| --------- | -------------------------------- |
| `--force` | Overwrite existing configuration |

```bash
audioshake config init
audioshake config init --force
```

##### config validate

Validate API key and connectivity.

```bash
audioshake config validate
```

**Output Example:**

```text
Validating configuration...
[OK] API key format valid
[OK] API connection successful
[OK] Account active (1,500 credits remaining)
```

---

### audioshake completion

Generate shell completions.

```text
audioshake completion <SHELL>
```

Available shells: `bash`, `zsh`, `fish`, `powershell`, `elvish`

```bash
# Bash
audioshake completion bash > ~/.local/share/bash-completion/completions/audioshake

# Zsh
audioshake completion zsh > ~/.zsh/completions/_audioshake

# Fish
audioshake completion fish > ~/.config/fish/completions/audioshake.fish

# PowerShell
audioshake completion powershell >> $PROFILE
```

---

## Exit Codes

| Code | Description                            |
| ---- | -------------------------------------- |
| 0    | Success                                |
| 1    | General error                          |
| 2    | Configuration error (missing API key)  |
| 3    | API error (authentication, rate limit) |
| 4    | Input error (invalid file, format)     |
| 5    | Network error (connection failed)      |
| 6    | Timeout error                          |

---

## Environment Variables

| Variable              | Description                                  |
| --------------------- | -------------------------------------------- |
| `AUDIOSHAKE_API_KEY`  | API key for authentication                   |
| `AUDIOSHAKE_BASE_URL` | Override API base URL                        |
| `NO_COLOR`            | Disable colored output                       |
| `RUST_LOG`            | Set logging level (debug, info, warn, error) |

---

## Supported Languages

The following ISO 639-1 language codes are supported for transcription:

| Code | Language             | Code | Language   |
| ---- | -------------------- | ---- | ---------- |
| en   | English              | es   | Spanish    |
| fr   | French               | de   | German     |
| it   | Italian              | pt   | Portuguese |
| nl   | Dutch                | pl   | Polish     |
| ru   | Russian              | ja   | Japanese   |
| zh   | Chinese              | ko   | Korean     |
| ar   | Arabic               | hi   | Hindi      |
| ...  | (73 languages total) |      |            |

See [AudioShake Supported Languages](https://developer.audioshake.ai/supported-languages) for the complete list.

---

## Examples

### Workflow: Album Stem Separation

```bash
#!/bin/bash
# Process entire album
for track in ./album/*.wav; do
    echo "Processing: $track"
    audioshake process "$track" \
        -m vocals -m instrumental \
        --hq --wait \
        -o "./stems/$(basename "$track" .wav)"
done
```

### Workflow: Podcast Transcription

```bash
#!/bin/bash
# Transcribe all episodes
for episode in ./episodes/*.mp3; do
    base=$(basename "$episode" .mp3)
    audioshake process "$episode" \
        -m transcription -m alignment \
        -f json --wait \
        -o "./transcripts"

    # Extract plain text
    jq -r '.words[].text' "./transcripts/${base}_transcription.json" > "./transcripts/${base}.txt"
done
```

### Workflow: CI/CD Integration

```yaml
# .github/workflows/process-audio.yml
name: Process Audio
on:
  workflow_dispatch:
    inputs:
      audio_url:
        description: 'Audio URL to process'
        required: true

jobs:
  process:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4

      - name: Install AudioShake CLI
        run: cargo install audioshake-cli

      - name: Process Audio
        env:
          AUDIOSHAKE_API_KEY: ${{ secrets.AUDIOSHAKE_API_KEY }}
        run: |
          audioshake process "${{ inputs.audio_url }}" \
            -m vocals -m instrumental \
            --wait -o ./output

      - name: Upload Results
        uses: actions/upload-artifact@v4
        with:
          name: stems
          path: ./output/*.wav
```

---

## Troubleshooting

### "API key not configured"

Ensure your API key is set via one of these methods:

1. `--api-key` flag
2. `AUDIOSHAKE_API_KEY` environment variable
3. `.env` file with `AUDIOSHAKE_API_KEY=...`
4. Config file at `~/.config/audioshake/config.toml`

### "Connection timeout"

1. Check your internet connection
2. Verify the API is accessible: `curl https://api.audioshake.ai/health`
3. Increase timeout: `--timeout 7200`

### "Rate limit exceeded"

The API has rate limits. Wait a few minutes and retry, or contact AudioShake support for higher limits.

### "Insufficient credits"

Check your credit balance in the AudioShake dashboard and purchase more if needed.

---

## References

- [AudioShake Developer Portal](https://developer.audioshake.ai/)
- [AudioShake Tasks API](https://developer.audioshake.ai/tasks)
- [AudioShake Getting Started](https://developer.audioshake.ai/tasksGettingStarted)
- [AudioShake Models](https://developer.audioshake.ai/models)
- [AudioShake Supported Formats](https://developer.audioshake.ai/supported-format)
- [AudioShake Supported Languages](https://developer.audioshake.ai/supported-languages)
