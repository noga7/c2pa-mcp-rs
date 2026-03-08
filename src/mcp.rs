//! MCP server wiring: tool handlers, result formatting, path normalization.

use std::collections::HashMap;
use std::path::PathBuf;

use mcp_protocol_sdk::prelude::*;

use crate::reader;
use crate::types::{C2PAReadError, C2PAResult};

// -----------------------------------------------------------------------------
// Path and result formatting
// -----------------------------------------------------------------------------

/// Normalize user input: strip `file://`, decode percent-encoding.
pub(crate) fn normalize_file_path(input: &str) -> PathBuf {
    let s = input.trim().strip_prefix("file://").unwrap_or(input.trim());
    let s = urlencoding::decode(s).unwrap_or(std::borrow::Cow::Borrowed(s));
    PathBuf::from(s.as_ref())
}

fn format_result(result: &C2PAResult) -> String {
    serde_json::to_string_pretty(result).unwrap_or_else(|_| format!("{result:?}"))
}

/// Turn a credential read result into the MCP tool result (shared by both tools).
pub(crate) fn to_tool_result(result: Result<C2PAResult, C2PAReadError>) -> ToolResult {
    let (c2pa_result, is_error) = match result {
        Ok(r) => (r, false),
        Err(e) => (C2PAResult::from_error(e.to_string()), true),
    };
    let text = format_result(&c2pa_result);
    let structured = serde_json::to_value(&c2pa_result).unwrap_or(serde_json::json!({}));
    ToolResult {
        content: vec![Content::text(text)],
        is_error: if is_error { Some(true) } else { None },
        structured_content: Some(structured),
        meta: None,
    }
}

// -----------------------------------------------------------------------------
// Tool handlers
// -----------------------------------------------------------------------------

pub struct ReadCredentialsFileHandler;

#[async_trait::async_trait]
impl ToolHandler for ReadCredentialsFileHandler {
    async fn call(&self, arguments: HashMap<String, serde_json::Value>) -> McpResult<ToolResult> {
        let file_path = arguments
            .get("filePath")
            .and_then(|v| v.as_str())
            .ok_or_else(|| McpError::Validation("Missing 'filePath' parameter".to_string()))?;
        let path = normalize_file_path(file_path);
        Ok(to_tool_result(reader::read_credentials_from_file(&path)))
    }
}

pub struct ReadCredentialsUrlHandler;

#[async_trait::async_trait]
impl ToolHandler for ReadCredentialsUrlHandler {
    async fn call(&self, arguments: HashMap<String, serde_json::Value>) -> McpResult<ToolResult> {
        let url = arguments
            .get("url")
            .and_then(|v| v.as_str())
            .ok_or_else(|| McpError::Validation("Missing 'url' parameter".to_string()))?;
        Ok(to_tool_result(reader::read_credentials_from_url(url).await))
    }
}
