# AudioShake CLI - Product Requirements Document

**Version:** 1.0.0
**Date:** January 2026
**Status:** Draft

---

## Executive Summary

The AudioShake CLI is a command-line interface tool that enables users to interact with the AudioShake API for audio processing tasks including stem separation, transcription, and audio analysis. Built in Rust for high performance and reliability, the CLI provides a seamless workflow for processing audio files from the terminal.

---

## Problem Statement

Audio professionals, developers, and content creators need a fast, scriptable way to:

- Separate audio stems (vocals, drums, bass, etc.) from mixed tracks
- Transcribe audio content with word-level alignment
- Extract dialogue from video/audio for post-production
- Batch process multiple files efficiently
- Integrate audio processing into automated workflows

Current solutions require web interfaces or complex API integrations. There is no native CLI tool for AudioShake that provides a streamlined developer experience.

---

## Goals

1. **Simplicity**: Single command to process audio files with sensible defaults
2. **Performance**: Async operations, parallel downloads, minimal latency
3. **Flexibility**: Support all AudioShake models and output formats
4. **Scriptability**: JSON/YAML output, exit codes, and pipe-friendly design
5. **Security**: Secure API key handling, no credentials in command history

---

## User Personas

### 1. Audio Engineer (Professional)

- **Needs**: High-quality stem separation for mixing/mastering
- **Workflow**: Batch process album tracks, extract stems for remixes
- **Technical Level**: Comfortable with terminal, values quality over speed

### 2. Podcast Producer

- **Needs**: Transcription with timestamps for show notes and chapters
- **Workflow**: Regular transcription of episodes, searchable archives
- **Technical Level**: Moderate, prefers simple commands

### 3. Video Post-Production

- **Needs**: Dialogue isolation, effects separation, music detection
- **Workflow**: Clean up audio tracks, prepare for ADR or mixing
- **Technical Level**: High, integrates into complex pipelines

### 4. Developer/DevOps

- **Needs**: API integration, automated processing, CI/CD integration
- **Workflow**: Build audio processing pipelines, batch operations
- **Technical Level**: Expert, needs scriptable interface

---

## User Journeys

### Journey 1: First-Time Setup

**Actor**: New user
**Goal**: Get started with AudioShake CLI

```text
1. User installs CLI: `cargo install audioshake-cli` or downloads binary
2. User creates .env file with API key from AudioShake dashboard
3. User validates setup: `audioshake config validate`
   → CLI confirms API key is valid and shows account status
4. User explores models: `audioshake models --detailed`
   → CLI displays available models with descriptions and costs
```

### Journey 2: Simple Stem Separation

**Actor**: Audio Engineer
**Goal**: Extract vocals from a song

```text
1. User has local MP3 file: song.mp3
2. User runs: `audioshake process song.mp3 -m vocals --wait`
3. CLI uploads file to AudioShake servers and submits task
4. CLI displays progress spinner while processing
5. CLI downloads vocals.wav to ./output/ directory
6. User has isolated vocal track ready for use
```

### Journey 3: Full Band Separation

**Actor**: Music Producer
**Goal**: Get all stems for remixing

```text
1. User runs: `audioshake process track.wav -m vocals -m drums -m bass -m guitar -m piano --hq --wait -o ./stems`
2. CLI processes with high-quality models (1.5 credits/min)
3. CLI downloads 5 stem files to ./stems/ directory
4. User has complete stem set for remix work
```

### Journey 4: Podcast Transcription

**Actor**: Podcast Producer
**Goal**: Get timestamped transcript for episode

```text
1. User runs: `audioshake process episode.mp3 -m transcription -m alignment -f json --wait`
2. CLI processes audio through transcription + alignment models
3. CLI downloads JSON with word-level timestamps
4. User imports JSON into podcast editing software for chapters
```

### Journey 5: Multi-Language Transcription

**Actor**: International Content Creator
**Goal**: Transcribe Spanish interview

```text
1. User runs: `audioshake process interview.mp3 -m transcription --language es -f txt --wait`
2. CLI specifies Spanish (es) language code for improved accuracy
3. CLI downloads plain text transcript
4. User has accurate Spanish transcription
```

### Journey 6: Video Dialogue Extraction

**Actor**: Post-Production Editor
**Goal**: Clean dialogue track from video

```text
1. User runs: `audioshake process scene.mp4 -m dialogue -m effects -f wav --wait`
2. CLI extracts audio from MP4 container
3. CLI separates dialogue from effects/ambience
4. CLI downloads separate dialogue.wav and effects.wav
5. Editor has clean tracks for mixing
```

### Journey 7: Batch Processing with Scripts

**Actor**: Developer
**Goal**: Process directory of audio files

```bash
#!/bin/bash
for file in ./audio/*.mp3; do
    audioshake process "$file" -m vocals -m instrumental --wait -o ./processed
done
```

### Journey 8: Async Task Management

**Actor**: DevOps Engineer
**Goal**: Submit jobs and check later

```text
1. Submit task: `audioshake process song.mp3 -m vocals`
   → CLI outputs: Task created: abc123-def456
2. Later, check status: `audioshake task status abc123-def456`
   → CLI shows: Status: processing (45%)
3. When complete: `audioshake task download abc123-def456 -o ./output`
   → CLI downloads all outputs
```

### Journey 9: CI/CD Integration

**Actor**: DevOps Engineer
**Goal**: Automated audio processing pipeline

```yaml
# GitHub Actions example
- name: Process Audio
  run: |
    audioshake process ${{ inputs.audio_url }} \
      -m vocals -m instrumental \
      --wait \
      -o json > result.json

- name: Upload Stems
  uses: actions/upload-artifact@v4
  with:
    path: ./output/*.wav
```

### Journey 10: Usage Monitoring

**Actor**: Team Lead
**Goal**: Track API usage and costs

```text
1. User runs: `audioshake task statistics`
2. CLI displays:
   - Total tasks: 150
   - Completed: 145
   - Failed: 5
   - Credits used: 1,234.5
   - Total duration: 823 minutes
```

---

## Use Cases

### UC-01: Process Local Audio File

**Preconditions**: Valid API key configured
**Input**: Local audio file path, model selection, output format
**Output**: Processed audio/text files in output directory

**Flow**:

1. CLI validates input file exists and format is supported
2. CLI uploads file to AudioShake servers (multipart upload)
3. CLI receives hosted URL for uploaded file
4. CLI submits task to AudioShake API with hosted URL
5. CLI polls for completion or uses webhook
6. CLI downloads results to output directory

### UC-02: Process Remote URL

**Preconditions**: Valid API key configured
**Input**: HTTPS URL to audio/video, model selection
**Output**: Processed audio/text files

**Flow**:

1. CLI validates URL is accessible HTTPS
2. CLI submits task directly to AudioShake API
3. CLI monitors task progress
4. CLI downloads results when complete

### UC-03: List Available Models

**Preconditions**: None (offline operation)
**Output**: Formatted list of models with capabilities

**Flow**:

1. CLI displays hardcoded model catalog
2. Optionally fetches current pricing from API

### UC-04: Monitor Task Progress

**Preconditions**: Valid task ID
**Output**: Task status with target progress

**Flow**:

1. CLI queries GET /tasks/{id}
2. CLI displays status for each target
3. CLI shows estimated time remaining if available

### UC-05: Download Task Results

**Preconditions**: Completed task ID
**Output**: Downloaded files

**Flow**:

1. CLI queries task for output URLs
2. CLI downloads each file with progress bar
3. CLI verifies file integrity
4. CLI organizes files by model type

---

## Functional Requirements

### FR-01: Authentication

- Read API key from `AUDIOSHAKE_API_KEY` environment variable
- Read API key from `.env` file in current directory
- Read API key from `~/.config/audioshake/config.toml`
- Accept API key via `--api-key` command line flag
- Never log or display API key in output

### FR-02: Input Handling

- Accept local file paths (MP3, WAV, FLAC, AAC, AIFF, MP4, MOV)
- Accept HTTPS URLs to remote audio/video
- Validate file format before processing
- Report clear error for unsupported formats

### FR-03: Local File Upload

- Upload local files directly to AudioShake servers via multipart form
- Support large files with appropriate timeout (10 minutes)
- Display upload progress indicator
- Receive hosted URL for use with Tasks API

### FR-04: Task Submission

- Submit to POST /tasks endpoint
- Support up to 10 targets per task (API limit)
- Support all model types: stems, dialogue, transcription
- Support high-quality model variants
- Support language specification for transcription

### FR-05: Task Monitoring

- Poll GET /tasks/{id} at configurable interval
- Display progress indicator (spinner or percentage)
- Support webhook callbacks for async notification
- Handle timeout gracefully with clear messaging

### FR-06: Result Download

- Download all output files from completed task
- Show download progress with file sizes
- Support resume of interrupted downloads
- Organize outputs by model type
- Support selective download (specific models only)

### FR-07: Output Formatting

- Default human-readable text output
- JSON output for scripting (`-o json`)
- YAML output for configuration (`-o yaml`)
- Quiet mode for minimal output
- Verbose mode for debugging

### FR-08: Configuration Management

- Initialize config file with `config init`
- Validate API key with `config validate`
- Set defaults with `config set`
- Display current config with `config show`

### FR-09: Shell Completions

- Generate completions for Bash, Zsh, Fish, PowerShell
- Install completions to appropriate system paths

---

## Non-Functional Requirements

### NFR-01: Performance

- Startup time < 100ms
- Memory usage < 50MB for typical operations
- Parallel downloads for multi-file results
- Async I/O throughout

### NFR-02: Reliability

- Graceful handling of network interruptions
- Automatic retry with exponential backoff
- Clear error messages with actionable guidance
- No data loss on unexpected termination

### NFR-03: Security

- API keys stored securely (not in command history)
- HTTPS only for all API communication
- Files uploaded directly to AudioShake (no third-party intermediaries)

### NFR-04: Portability

- Support macOS (Intel and Apple Silicon)
- Support Linux (x86_64, aarch64)
- Support Windows (x86_64)
- Single static binary with no runtime dependencies

### NFR-05: Usability

- Comprehensive `--help` for all commands
- Colored output for terminals that support it
- Progress indicators for long operations
- Example commands in documentation

---

## Success Metrics

1. **Adoption**: 1000+ downloads in first 3 months
2. **Reliability**: 99.9% successful task completion rate
3. **Performance**: Average task submission < 2 seconds
4. **Satisfaction**: Positive feedback on GitHub issues

---

## Out of Scope

- GUI or TUI interface (CLI only)
- Real-time audio streaming
- Audio format conversion (use ffmpeg separately)
- Account management (use web dashboard)
- Billing management (use web dashboard)

---

## Dependencies

### Required

- Rust 1.92+ (for async traits, 2024 edition features)
- Internet connection (for API access)

### Optional

- ffmpeg (for format conversion, not handled by CLI)

---

## Timeline

| Phase | Description                           | Duration |
| ----- | ------------------------------------- | -------- |
| 1     | Core infrastructure (CLI, API client) | -        |
| 2     | Main commands (process, task)         | -        |
| 3     | Advanced features (upload, progress)  | -        |
| 4     | Polish (tests, docs, release)         | -        |

---

## References

- [AudioShake Developer Portal](https://developer.audioshake.ai/)
- [AudioShake Tasks API](https://developer.audioshake.ai/tasks)
- [AudioShake Models](https://developer.audioshake.ai/models)
- [AudioShake Supported Formats](https://developer.audioshake.ai/supported-format)
- [AudioShake Supported Languages](https://developer.audioshake.ai/supported-languages)
