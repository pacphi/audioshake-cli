# ADR-002: Cloud Storage Upload Strategy

## Status

**Proposed** - January 2026

## Context

The AudioShake CLI needs to support processing local audio/video files. The AudioShake Tasks API (`api.audioshake.ai/tasks`) only accepts publicly accessible HTTPS URLs - it does not support direct file uploads.

### Previous Approaches

**ADR-001: Cloudflare Quick Tunnels** (Rejected)

- Started local HTTP server + cloudflared tunnel
- Failed due to tunnel unreliability, race conditions, and API fetch failures
- Cloudflare explicitly states Quick Tunnels have "no uptime guarantee"

**ADR-001 Decision: Legacy Direct Upload** (Now Deprecated)

- Used `https://groovy.audioshake.ai/upload` endpoint
- Worked initially but now returns `401 Unauthorized: Invalid Token`
- The legacy API requires a different authentication token than the main Tasks API
- AudioShake documentation marks this as "legacy"

### Current Problem

Users with local files cannot process them because:

1. Legacy upload API rejects the standard API key
2. Tunnel approaches are unreliable
3. Tasks API requires publicly accessible URLs

## Decision

Implement **cloud storage integration** using **Cloudflare R2** as the primary provider, with the **AWS SDK for Rust** (`aws-sdk-s3`) for S3-compatible operations.

### Architecture

```
┌─────────────┐     ┌──────────────────┐     ┌─────────────────┐
│ Local File  │────▶│ Cloudflare R2    │────▶│ AudioShake      │
│ song.mp3    │     │ (User's Bucket)  │     │ Tasks API       │
└─────────────┘     └──────────────────┘     └─────────────────┘
      │                     │                        │
      │ CLI uploads         │ Public URL             │ Fetches file
      │ via S3 API          │ https://bucket         │ from R2 CDN
      │                     │ .r2.dev/key            │
      └─────────────────────┴────────────────────────┘
```

### Flow

1. User configures R2 credentials once (stored in config file)
2. User runs `audioshake process local-file.mp3 --model vocals`
3. CLI detects local file, uploads to user's R2 bucket
4. CLI constructs public URL from bucket configuration
5. CLI submits URL to AudioShake Tasks API
6. AudioShake fetches file from R2 (fast, reliable CDN)
7. Processing proceeds normally

### Why Cloudflare R2

| Criteria         | R2           | AWS S3     | Backblaze B2 |
| ---------------- | ------------ | ---------- | ------------ |
| Egress cost      | **Free**     | $0.09/GB   | $0.01/GB     |
| Free tier        | 10GB storage | 5GB (12mo) | 10GB         |
| S3 compatible    | Yes          | Native     | Yes          |
| CDN included     | Yes (global) | Extra cost | No           |
| Setup complexity | Low          | Medium     | Low          |

**Key advantage**: Zero egress fees. When AudioShake fetches files from R2, it costs the user nothing. This is critical for audio/video files that can be 100MB+.

### Rust SDK Choice: `aws-sdk-s3`

Selected based on [Cloudflare's official recommendation](https://developers.cloudflare.com/r2/examples/aws/aws-sdk-rust/):

**Pros:**

- Official AWS SDK, well-maintained by Amazon
- First-class async/tokio support
- Full S3 API coverage including multipart uploads
- Cloudflare provides explicit R2 configuration examples
- Large community and ecosystem

**Cons:**

- Larger dependency footprint than lightweight alternatives
- Requires careful configuration for R2 endpoint

**Alternatives considered:**

- `rust-s3`: Lighter but less maintained
- `cf-r2-sdk`: R2-specific but smaller community
- Custom reqwest implementation: More work, less reliable

### Configuration Schema

```toml
# ~/.config/audioshake/config.toml

[storage]
provider = "r2"                          # Future: could support "s3", "b2"
bucket = "my-audioshake-uploads"
public_url_base = "https://my-audioshake-uploads.r2.dev"  # or custom domain

[storage.r2]
account_id = "abc123..."
access_key_id = "..."
secret_access_key = "..."                # Stored securely
```

Environment variable overrides:

```bash
AUDIOSHAKE_STORAGE_PROVIDER=r2
AUDIOSHAKE_STORAGE_BUCKET=my-bucket
AUDIOSHAKE_R2_ACCOUNT_ID=abc123
AUDIOSHAKE_R2_ACCESS_KEY_ID=xxx
AUDIOSHAKE_R2_SECRET_ACCESS_KEY=xxx
AUDIOSHAKE_STORAGE_PUBLIC_URL_BASE=https://my-bucket.r2.dev
```

### Public URL Strategy

R2 offers two public access methods:

1. **r2.dev subdomain** (development)
   - Format: `https://{bucket}.r2.dev/{key}`
   - Rate limited (hundreds req/sec)
   - No setup required beyond enabling in dashboard

2. **Custom domain** (production)
   - Format: `https://files.example.com/{key}`
   - No rate limits
   - Enables caching, WAF, analytics

For CLI users, r2.dev is sufficient. The rate limit won't be hit for typical processing workloads.

### Object Naming Strategy

```
{timestamp}-{random}-{original_filename}
```

Example: `20260110-a7f3b2c1-song.mp3`

Benefits:

- Unique keys prevent collisions
- Timestamp enables lifecycle cleanup
- Original filename preserved for debugging

### Lifecycle Management

Configure R2 bucket with object lifecycle rules:

- Auto-delete objects after 7 days
- Multipart uploads expire after 24 hours (R2 default: 7 days)

This ensures temporary processing files don't accumulate and cost money.

### Implementation Modules

```
src/
├── storage/
│   ├── mod.rs           # Storage trait and provider selection
│   ├── r2.rs            # R2-specific implementation
│   └── config.rs        # Storage configuration types
├── config/
│   └── types.rs         # Add StorageConfig
└── commands/
    └── process.rs       # Update to use storage module
```

### API Surface

```rust
/// Storage provider trait for future extensibility
#[async_trait]
pub trait StorageProvider: Send + Sync {
    /// Upload a file and return its public URL
    async fn upload(&self, path: &Path) -> Result<Url>;

    /// Delete an object by key (optional cleanup)
    async fn delete(&self, key: &str) -> Result<()>;
}

/// R2 implementation
pub struct R2Storage {
    client: aws_sdk_s3::Client,
    bucket: String,
    public_url_base: Url,
}
```

### Error Handling

New error variants:

```rust
pub enum StorageError {
    NotConfigured,           // No storage provider configured
    InvalidCredentials,      // Auth failed
    BucketNotFound,          // Bucket doesn't exist
    UploadFailed(String),    // Upload-specific errors
    PublicAccessDisabled,    // Bucket not public
}
```

### User Experience

**First-time setup:**

```bash
# Interactive configuration
$ audioshake config storage

Storage provider: r2
R2 Account ID: abc123...
R2 Access Key ID: xxx
R2 Secret Access Key: ********
Bucket name: my-audioshake-uploads
Public URL base: https://my-audioshake-uploads.r2.dev

Testing connection... OK
Verifying public access... OK
Configuration saved to ~/.config/audioshake/config.toml
```

**Daily usage (unchanged):**

```bash
$ audioshake process song.mp3 --model vocals
Uploading song.mp3 to R2... done (3.2s)
Creating task with 1 target(s)
Task created: task_abc123
Processing... completed
✓ Downloaded vocals.mp3
```

### Fallback Behavior

If storage is not configured and user provides local file:

```
Error: Local file processing requires cloud storage configuration.

To process local files, configure a storage provider:
  audioshake config storage

Or provide a publicly accessible URL:
  audioshake process https://example.com/song.mp3 --model vocals
```

### Security Considerations

1. **Credentials storage**: Use `secrecy` crate (already in use) for secret access key
2. **Bucket permissions**: Document that bucket must be public (read) but credentials should only allow write
3. **Object keys**: Include randomness to prevent guessing
4. **Lifecycle**: Auto-delete prevents data accumulation

### Migration Path

1. Mark legacy `upload_file` method as `#[deprecated]`
2. Add warning when legacy upload is attempted
3. Document new R2 setup process
4. Keep legacy code for users who obtain valid legacy tokens

## Consequences

### Positive

- **Reliability**: R2 is production infrastructure with SLA, unlike tunnels or legacy API
- **Cost-effective**: Zero egress fees, generous free tier covers most users
- **Performance**: Global CDN ensures fast fetches from any AudioShake region
- **User control**: Users own their storage, can inspect/manage uploaded files
- **Extensibility**: StorageProvider trait enables future S3/B2/GCS support

### Negative

- **Setup overhead**: One-time R2 account and bucket configuration required
- **Cloudflare dependency**: Ties users to Cloudflare (mitigated by trait abstraction)
- **Credentials management**: Users must securely manage R2 API tokens

### Neutral

- **Same end result**: Files are still uploaded before processing, just to a different location
- **Similar latency**: Upload to R2 + fetch by AudioShake ≈ upload to legacy API

## Alternatives Considered

### 1. Fix Legacy API Authentication

- Requires AudioShake to issue different tokens
- Out of our control
- Legacy API may be deprecated entirely

### 2. Presigned Upload URLs from AudioShake

- Would require AudioShake to implement new API endpoint
- Cleanest solution but requires vendor cooperation
- Could propose as future enhancement

### 3. ngrok/Tunnel Integration

- Same reliability issues as cloudflared
- External dependency (ngrok binary)
- Free tier limitations

### 4. Require Users to Host Files

- Poor UX for local file workflows
- Shifts complexity to user
- Valid fallback but not primary solution

## Implementation Plan

1. Add `aws-sdk-s3` and related dependencies to Cargo.toml
2. Create `src/storage/` module with trait and R2 implementation
3. Extend configuration schema and loader
4. Add `audioshake config storage` subcommand
5. Update `process` command to use storage module
6. Mark legacy upload as deprecated
7. Update documentation (README, DEVELOPER.md, help text)
8. Add integration tests with mocked S3 endpoint

## References

- [Cloudflare R2 Documentation](https://developers.cloudflare.com/r2/)
- [AWS SDK for Rust with R2](https://developers.cloudflare.com/r2/examples/aws/aws-sdk-rust/)
- [R2 S3 API Compatibility](https://developers.cloudflare.com/r2/api/s3/api/)
- [R2 Public Buckets](https://developers.cloudflare.com/r2/buckets/public-buckets/)
- [R2 Object Lifecycles](https://developers.cloudflare.com/r2/buckets/object-lifecycles/)
- [R2 Pricing](https://developers.cloudflare.com/r2/pricing/)
- [R2 Limits](https://developers.cloudflare.com/r2/platform/limits/)
- [ADR-001: Local File Upload Strategy](./001-local-file-upload-strategy.md)
