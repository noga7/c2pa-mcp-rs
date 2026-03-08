#!/usr/bin/env bash
# Setup script for c2pa-mcp-rs: installs Rust (if needed), builds the server,
# checks for TrustMark models, and prints your MCP config.
# Run from this repo: ./setup.sh

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
cd "$SCRIPT_DIR"

BINARY_NAME="c2pa-mcp-rs"
RELEASE_BINARY="$SCRIPT_DIR/target/release/$BINARY_NAME"
MODELS_DIR="$SCRIPT_DIR/models"

echo "=============================================="
echo "  c2pa-mcp-rs setup"
echo "=============================================="
echo ""

# ---------------------------------------------------------------------------
# Step 1: Rust (required)
# ---------------------------------------------------------------------------
echo "Step 1: Checking for Rust..."
if command -v rustc >/dev/null 2>&1 && command -v cargo >/dev/null 2>&1; then
    echo "  ✓ Rust is installed: $(rustc --version)"
else
    echo "  Rust was not found. It is required to build the MCP server."
    echo ""
    echo "  Install Rust by running this command, then restart your terminal and run this setup again:"
    echo ""
    echo "    curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh"
    echo ""
    if [[ "$OSTYPE" == "darwin"* ]]; then
        echo "  (On macOS you can also install Xcode Command Line Tools first if prompted: xcode-select --install)"
    fi
    exit 1
fi
echo ""

# ---------------------------------------------------------------------------
# Step 2: Build the server
# ---------------------------------------------------------------------------
echo "Step 2: Building the MCP server (this may take a few minutes the first time)..."
if ! cargo build --release 2>&1; then
    echo ""
    echo "  ✗ Build failed. Check the messages above for errors."
    exit 1
fi
echo "  ✓ Build succeeded."
echo ""

# ---------------------------------------------------------------------------
# Step 3: TrustMark models (required for watermark detection)
# ---------------------------------------------------------------------------
echo "Step 3: Checking TrustMark models..."
if [[ -d "$MODELS_DIR" ]] && [[ -n "$(ls -A "$MODELS_DIR" 2>/dev/null)" ]]; then
    echo "  ✓ TrustMark models directory found: $MODELS_DIR"
else
    echo "  TrustMark models directory not found or empty."
    echo ""
    echo "  Watermark detection needs ONNX model files. To get them:"
    echo "  1. See: https://opensource.contentauthenticity.org/docs/trustmark/rust/"
    echo "  2. From the TrustMark Rust workspace, run: cargo xtask fetch-models"
    echo "  3. Copy the downloaded 'models' directory into this repo, or set TRUSTMARK_MODELS to its path."
    echo ""
    echo "  Embedded C2PA will still work without models; only watermark fallback will be skipped."
fi
echo ""

# ---------------------------------------------------------------------------
# Step 4: Success and config
# ---------------------------------------------------------------------------
if [[ ! -x "$RELEASE_BINARY" ]]; then
    echo "  ✗ Binary not found at $RELEASE_BINARY"
    exit 1
fi

echo "=============================================="
echo "  Setup complete"
echo "=============================================="
echo ""
echo "Your MCP server binary is at:"
echo "  $RELEASE_BINARY"
echo ""
echo "Add it to your MCP client (e.g. Claude Desktop) by putting this in your config:"
echo ""
echo "  \"content-credentials\": {"
echo "    \"command\": \"$RELEASE_BINARY\""
echo "  }"
echo ""
if [[ ! -d "$MODELS_DIR" ]] || [[ -z "$(ls -A "$MODELS_DIR" 2>/dev/null)" ]]; then
    echo "To enable TrustMark watermark detection, add the models path:"
    echo ""
    echo "  \"content-credentials\": {"
    echo "    \"command\": \"$RELEASE_BINARY\","
    echo "    \"env\": {"
    echo "      \"TRUSTMARK_MODELS\": \"/path/to/models\""
    echo "    }"
    echo "  }"
    echo ""
fi
echo "Config file locations:"
echo "  • macOS (Claude Desktop): ~/Library/Application Support/Claude/claude_desktop_config.json"
echo "  • Cursor: see Cursor MCP settings"
echo ""
echo "Restart your MCP client after changing the config."
echo ""
