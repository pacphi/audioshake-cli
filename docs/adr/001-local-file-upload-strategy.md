# ADR-001: Local File Upload Strategy

## Status

**Superseded** by [ADR-002](./002-cloud-storage-upload-strategy.md) - January 2026

_Originally Accepted - January 2026_

## Context

The AudioShake CLI needs to support processing local audio/video files. The AudioShake Tasks API (`api.audioshake.ai/tasks`) only accepts URLs - it does not support direct file uploads. This means local files must be made accessible via a public URL before they can be processed.

### Initial Approach: Cloudflare Quick Tunnels

The original implementation used [Cloudflare Quick Tunnels](https://developers.cloudflare.com/cloudflare-one/connections/connect-networks/do-more-with-tunnels/trycloudflare/) (TryCloudflare) to expose local files:

1. Start a local Axum HTTP server serving the file at a random port
2. Start `cloudflared tunnel --url http://localhost:PORT` to create a public tunnel
3. Parse the generated `*.trycloudflare.com` URL from cloudflared's stderr
4. Submit this URL to the AudioShake Tasks API
5. Keep the tunnel running while the API fetches and processes the file

This approach required users to install `cloudflared` as a dependency.

## Problem

The cloudflared tunnel approach proved unreliable in production. Multiple issues were encountered:

### Issue 1: Race Condition on Tunnel URL Detection

Cloudflared logs the tunnel URL to stderr _before_ the tunnel is fully established:

```
INF Your quick Tunnel has been created! Visit it at (it may take some time to be reachable):
INF https://xxx.trycloudflare.com
...
INF Registered tunnel connection connIndex=0
```

The initial implementation returned the URL immediately after detecting it, but the tunnel wasn't ready until "Registered tunnel connection" appeared.

**Attempted fix**: Wait for "Registered tunnel connection" before proceeding.

### Issue 2: Stderr Pipe Blocking

After extracting the tunnel URL, the code stopped reading from cloudflared's stderr. Since cloudflared continues writing logs, the pipe buffer would fill up, causing cloudflared to block and preventing the tunnel from fully establishing.

**Attempted fix**: Spawn a background task to continuously drain stderr.

### Issue 3: QUIC Protocol Issues

Some ISPs have issues with QUIC (the default protocol). Users experienced timeouts and connection failures.

**Attempted fix**: Add `--protocol http2` flag to force HTTP/2 instead of QUIC.

### Issue 4: Local Verification Failures

Even after the tunnel was registered, verification requests from the local machine would fail:

```
Attempt 1/20: error sending request for url (https://xxx.trycloudflare.com/...)
Attempt 2/20: error sending request for url (https://xxx.trycloudflare.com/...)
...
```

The tunnel was registered with Cloudflare's edge, but the local machine couldn't reach it (likely DNS propagation or network-specific issues). However, external servers (like AudioShake's) might have had better connectivity.

**Attempted fix**: Remove local verification and trust the "Registered tunnel connection" message.

### Issue 5: Persistent API Fetch Failures

Even after all fixes, the AudioShake API consistently returned:

```json
{ "statusCode": 400, "error": "Bad Request", "message": "Error fetching or processing asset" }
```

This indicated that even when the tunnel appeared registered, AudioShake's servers couldn't reliably fetch files from the quick tunnel.

### Root Cause Analysis

Cloudflare explicitly states that Quick Tunnels have ["no uptime guarantee"](https://developers.cloudflare.com/cloudflare-one/connections/connect-networks/do-more-with-tunnels/trycloudflare/). They are designed for quick experiments and demos, not production use. The inherent unreliability of quick tunnels made them unsuitable for a CLI tool that needs to work consistently.

## Decision

Replace the cloudflared tunnel approach with **direct file upload** to AudioShake's servers using the legacy API endpoint.

### New Architecture

1. For local files: Upload directly to `https://groovy.audioshake.ai/upload` using multipart/form-data
2. The upload returns a hosted URL (the `link` field in the response)
3. Use this hosted URL with the Tasks API

### Implementation Details

**API Client Addition** (`src/api/client.rs`):

```rust
pub async fn upload_file(&self, path: &Path) -> Result<UploadResponse> {
    // Read file, determine MIME type
    // Create multipart form with file
    // POST to https://groovy.audioshake.ai/upload
    // Return response containing hosted URL
}
```

**Process Command Change** (`src/commands/process.rs`):

```rust
// Before: Start tunnel, get tunnel URL, submit to API
// After: Upload file, get hosted URL, submit to API
```

### Removed Components

- `src/tunnel/mod.rs` - Entire tunnel module deleted
- `TunnelError` enum - Removed from error types
- cloudflared dependency - No longer required

## Consequences

### Positive

- **Reliability**: Direct upload to AudioShake's servers eliminates all tunnel-related failure modes
- **Simplicity**: No external dependencies (cloudflared) required
- **Predictability**: Upload progress is visible and deterministic
- **Security**: Files go directly to AudioShake rather than through a third-party tunnel

### Negative

- **Upload time**: Large files must be fully uploaded before processing begins (vs. streaming through tunnel)
- **Bandwidth**: File data is uploaded to AudioShake's servers rather than fetched on-demand
- **Legacy API dependency**: Uses the legacy upload endpoint rather than the new Tasks API

### Neutral

- **Same end result**: Both approaches result in AudioShake having access to the file for processing

## Alternatives Considered

1. **Named Cloudflare Tunnels**: Requires Cloudflare account and setup; too complex for a CLI tool
2. **ngrok or other tunnel services**: Same reliability concerns as quick tunnels
3. **Require users to host files**: Poor user experience for local file processing
4. **S3/cloud storage intermediary**: Adds complexity and cloud provider dependency

## References

- [Cloudflare Quick Tunnels documentation](https://developers.cloudflare.com/cloudflare-one/connections/connect-networks/do-more-with-tunnels/trycloudflare/)
- [AudioShake Legacy File Upload API](https://developer.audioshake.ai/legacy-api/file-upload)
- [AudioShake Tasks API](https://developer.audioshake.ai/tasksGettingStarted)
