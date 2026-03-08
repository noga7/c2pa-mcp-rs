//! MCP Content Credentials Server (Rust)
//!
//! Reads C2PA Content Credentials from images and media: embedded manifest first,
//! then TrustMark watermark when available. Exposes MCP tools over stdio.
//!
//! # Module layout (review order)
//!
//! - [`types`] — API types and result constructors
//! - [`trustmark`] — TrustMark watermark decoding (Rust crate, cached)
//! - [`reader`] — Orchestration: file/URL → C2PA then TrustMark
//! - [`mcp`] — Tool handlers and result formatting for MCP

mod mcp;
mod reader;
mod trustmark;
mod types;

use mcp_protocol_sdk::prelude::*;
use mcp_protocol_sdk::transport::StdioServerTransport;
use serde_json::json;

use mcp::{ReadCredentialsFileHandler, ReadCredentialsUrlHandler};

const SERVER_NAME: &str = "content-credentials";
const SERVER_VERSION: &str = "0.1.0";

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let mut server = McpServer::new(SERVER_NAME.to_string(), SERVER_VERSION.to_string());

    server
        .add_tool(
            "read_credentials_file".to_string(),
            Some(
                "Read Content Credentials from a local file. USE THIS TOOL when the user drops a file or provides a file path AND asks about who made it, how it was made, whether it has Content Credentials, or mentions c2pa. Path can be absolute, relative, or file:// URI.".to_string(),
            ),
            json!({
                "type": "object",
                "properties": {
                    "filePath": {
                        "type": "string",
                        "description": "Path to the file (absolute, relative, or file:// URI)"
                    }
                },
                "required": ["filePath"]
            }),
            ReadCredentialsFileHandler,
        )
        .await?;

    server
        .add_tool(
            "read_credentials_url".to_string(),
            Some(
                "Read Content Credentials from a file at a URL. USE THIS TOOL when the user provides a URL and asks about credentials, provenance, or c2pa. Downloads the file temporarily, checks for C2PA manifest, then cleans up.".to_string(),
            ),
            json!({
                "type": "object",
                "properties": {
                    "url": {
                        "type": "string",
                        "description": "HTTP or HTTPS URL of the file"
                    }
                },
                "required": ["url"]
            }),
            ReadCredentialsUrlHandler,
        )
        .await?;

    let transport = StdioServerTransport::new();
    server.start(transport).await?;
    Ok(())
}
