//! Local file server with tunnel support

use std::path::PathBuf;
use std::process::Stdio;

use axum::routing::get_service;
use axum::Router;
use tokio::io::{AsyncBufReadExt, BufReader};
use tokio::net::TcpListener;
use tokio::process::{Child, Command};
use tower_http::services::ServeFile;
use tracing::{debug, info, warn};
use url::Url;
use uuid::Uuid;

use crate::error::{Result, TunnelError};

/// Local file server with cloudflared tunnel
pub struct TunnelService {
    file_path: PathBuf,
    server_handle: Option<tokio::task::JoinHandle<()>>,
    tunnel_process: Option<Child>,
    public_url: Option<Url>,
    port: u16,
    token: String,
}

impl TunnelService {
    /// Create a new tunnel service for a local file
    #[must_use]
    pub fn new(file_path: PathBuf) -> Self {
        Self {
            file_path,
            server_handle: None,
            tunnel_process: None,
            public_url: None,
            port: 0,
            token: Uuid::new_v4().to_string(),
        }
    }

    /// Start the local server and tunnel
    ///
    /// # Errors
    ///
    /// Returns an error if the server or tunnel fails to start.
    pub async fn start(&mut self) -> Result<Url> {
        // Start local server
        self.start_server().await?;

        // Start tunnel
        self.start_tunnel().await?;

        self.public_url
            .clone()
            .ok_or_else(|| TunnelError::UrlParseFailed.into())
    }

    /// Stop the tunnel and server
    pub async fn stop(&mut self) {
        // Stop tunnel
        if let Some(mut process) = self.tunnel_process.take() {
            let _ = process.kill().await;
            let _ = process.wait().await;
            info!("Tunnel stopped");
        }

        // Stop server
        if let Some(handle) = self.server_handle.take() {
            handle.abort();
            info!("Local server stopped");
        }
    }

    async fn start_server(&mut self) -> Result<()> {
        let file_path = self.file_path.clone();
        let token = self.token.clone();

        // Create router with single file endpoint
        let app = Router::new().route(
            &format!("/{token}"),
            get_service(ServeFile::new(&file_path)),
        );

        // Bind to random available port
        let listener = TcpListener::bind("127.0.0.1:0").await?;
        let addr = listener.local_addr()?;
        self.port = addr.port();

        info!("Local server started on port {}", self.port);

        // Spawn server task
        let handle = tokio::spawn(async move {
            if let Err(e) = axum::serve(listener, app).await {
                warn!("Server error: {}", e);
            }
        });

        self.server_handle = Some(handle);
        Ok(())
    }

    async fn start_tunnel(&mut self) -> Result<()> {
        // Check if cloudflared is available
        if !Self::is_cloudflared_available().await {
            return Err(TunnelError::CloudflaredNotFound.into());
        }

        info!("Starting cloudflared tunnel for port {}", self.port);

        let mut child = Command::new("cloudflared")
            .args([
                "tunnel",
                "--url",
                &format!("http://localhost:{}", self.port),
                "--no-autoupdate",
            ])
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .map_err(|e| TunnelError::StartFailed(e.to_string()))?;

        // Parse the tunnel URL from stderr
        let stderr = child
            .stderr
            .take()
            .ok_or_else(|| TunnelError::StartFailed("No stderr".into()))?;

        let mut reader = BufReader::new(stderr).lines();

        // Wait for tunnel URL with timeout
        let url = tokio::time::timeout(std::time::Duration::from_secs(30), async {
            while let Ok(Some(line)) = reader.next_line().await {
                debug!("cloudflared: {}", line);
                if let Some(url) = Self::extract_tunnel_url(&line) {
                    return Ok(url);
                }
            }
            Err(TunnelError::Terminated)
        })
        .await
        .map_err(|_| TunnelError::StartFailed("Timeout waiting for tunnel URL".into()))??;

        // Construct full URL with token
        let full_url = url.join(&self.token)?;

        self.tunnel_process = Some(child);
        self.public_url = Some(full_url.clone());

        info!("Tunnel started: {}", full_url);
        Ok(())
    }

    async fn is_cloudflared_available() -> bool {
        Command::new("cloudflared")
            .arg("--version")
            .output()
            .await
            .is_ok()
    }

    fn extract_tunnel_url(line: &str) -> Option<Url> {
        // cloudflared outputs: "... https://xxx.trycloudflare.com ..."
        line.split_whitespace()
            .find(|word| {
                word.starts_with("https://") && word.contains("trycloudflare.com")
            })
            .and_then(|s| Url::parse(s).ok())
    }
}

impl Drop for TunnelService {
    fn drop(&mut self) {
        if let Some(mut process) = self.tunnel_process.take() {
            let _ = process.start_kill();
        }
        if let Some(handle) = self.server_handle.take() {
            handle.abort();
        }
    }
}
