//! Shared types for the Content Credentials API.
//!
//! Matches the TypeScript MCP server's response shape for compatibility.

use serde::Serialize;
use thiserror::Error;

/// TrustMark watermark payload; matches TypeScript `TrustMarkWatermarkData`.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TrustMarkWatermarkData {
    pub identifier: String,
    pub schema: String,
    pub raw: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub manifest_url: Option<String>,
}

/// Result returned to MCP clients; matches the TypeScript server's `C2PAResult` shape.
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct C2PAResult {
    pub success: bool,
    pub has_credentials: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub manifest_data: Option<serde_json::Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub trust_mark_data: Option<TrustMarkWatermarkData>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub raw_output: Option<String>,
}

impl C2PAResult {
    /// Embedded C2PA manifest was found.
    pub(crate) fn embedded(detailed_json: String) -> Self {
        let manifest_data = serde_json::from_str(&detailed_json)
            .unwrap_or_else(|_| serde_json::json!({ "raw": detailed_json }));
        Self {
            success: true,
            has_credentials: true,
            manifest_data: Some(manifest_data),
            trust_mark_data: None,
            error: None,
            raw_output: Some(detailed_json),
        }
    }

    /// TrustMark watermark was found (no embedded manifest).
    pub(crate) fn trust_mark(trust_mark_data: TrustMarkWatermarkData) -> Self {
        Self {
            success: true,
            has_credentials: true,
            manifest_data: None,
            trust_mark_data: Some(trust_mark_data),
            error: None,
            raw_output: None,
        }
    }

    /// No credentials found (neither embedded nor watermark).
    pub(crate) fn none() -> Self {
        Self {
            success: true,
            has_credentials: false,
            manifest_data: None,
            trust_mark_data: None,
            error: None,
            raw_output: None,
        }
    }

    /// Read failed (IO, network, or invalid input).
    pub fn from_error(msg: impl Into<String>) -> Self {
        Self {
            success: false,
            has_credentials: false,
            manifest_data: None,
            trust_mark_data: None,
            error: Some(msg.into()),
            raw_output: None,
        }
    }
}

/// Errors that can occur when reading credentials.
#[derive(Error, Debug)]
pub enum C2PAReadError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    #[error("C2PA error: {0}")]
    #[allow(dead_code)]
    C2PA(String),
    #[error("Download error: {0}")]
    Download(#[from] reqwest::Error),
    #[error("Invalid URL or path")]
    InvalidInput,
}
