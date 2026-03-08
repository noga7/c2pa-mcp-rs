# c2pa-mcp-rs

MCP server for reading **Content Credentials (C2PA)** from images and media. Embedded C2PA via the [c2pa](https://crates.io/crates/c2pa) crate, watermark fallback via the [TrustMark](https://docs.rs/trustmark/latest/trustmark/) crate. Works best for:
- Inspecting assets via URL
- Providing a path to a locally stored file

## Install

```bash
cd /path/to/c2pa-mcp-rs
./setup.sh
```

If Rust isn’t installed, the script prints the install command; restart your terminal and run `./setup.sh` again.

**Manual:** Install [Rust](https://rustup.rs/) 1.88+, then `cargo build --release`. Binary: `target/release/c2pa-mcp-rs`. For watermark detection, put TrustMark ONNX models in `models/` or set `TRUSTMARK_MODELS`; see [CAI TrustMark Rust](https://opensource.contentauthenticity.org/docs/trustmark/rust/).

## Config

Add to your MCP client config (e.g. Claude Desktop: `~/Library/Application Support/Claude/claude_desktop_config.json`):

```json
{
  "mcpServers": {
    "content-credentials": {
      "command": "/path/to/c2pa-mcp-rs/target/release/c2pa-mcp-rs"
    }
  }
}
```

Use the full path to the binary. To point at a custom models dir: add `"env": { "TRUSTMARK_MODELS": "/path/to/models" }`. Restart the client after editing config.

## Tools

| Tool | Description |
|------|-------------|
| `read_credentials_file` | Read credentials from a local path (absolute, relative, or `file://`). |
| `read_credentials_url`  | Read credentials from a URL (downloads temporarily, then inspects). |

Response: `success`, `hasCredentials`, optional `manifestData`, optional `trustMarkData`, `error`, `rawOutput` (camelCase JSON).

## Links

[C2PA Rust SDK](https://opensource.contentauthenticity.org/docs/rust-sdk/) · [TrustMark Rust](https://opensource.contentauthenticity.org/docs/trustmark/rust/) · [MCP](https://modelcontextprotocol.io/)

**License:** MIT OR Apache-2.0
