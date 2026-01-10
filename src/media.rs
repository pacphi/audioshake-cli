//! Media file utilities

use std::path::Path;
use std::process::Command;

use crate::error::{AppError, Result};

/// Get the duration of a media file in seconds using ffprobe
///
/// # Errors
///
/// Returns an error if ffprobe is not installed or fails to parse the file
pub fn get_duration(path: &Path) -> Result<f64> {
    let output = Command::new("ffprobe")
        .args([
            "-v",
            "error",
            "-show_entries",
            "format=duration",
            "-of",
            "default=noprint_wrappers=1:nokey=1",
        ])
        .arg(path)
        .output()
        .map_err(|e| {
            if e.kind() == std::io::ErrorKind::NotFound {
                AppError::User(
                    "ffprobe not found. Install FFmpeg to use --dry-run cost estimation."
                        .to_string(),
                )
            } else {
                AppError::User(format!("Failed to run ffprobe: {e}"))
            }
        })?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(AppError::User(format!(
            "ffprobe failed to read file: {stderr}"
        )));
    }

    let duration_str = String::from_utf8_lossy(&output.stdout);
    duration_str
        .trim()
        .parse::<f64>()
        .map_err(|e| AppError::User(format!("Failed to parse duration: {e}")))
}

/// Format duration in seconds as human-readable string (MM:SS)
#[must_use]
#[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
pub fn format_duration(seconds: f64) -> String {
    let total_seconds = seconds.round() as u64;
    let minutes = total_seconds / 60;
    let secs = total_seconds % 60;
    format!("{minutes}:{secs:02}")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_format_duration() {
        assert_eq!(format_duration(0.0), "0:00");
        assert_eq!(format_duration(59.0), "0:59");
        assert_eq!(format_duration(60.0), "1:00");
        assert_eq!(format_duration(125.5), "2:06");
        assert_eq!(format_duration(3661.0), "61:01");
    }
}
