//! Orchestrates reading Content Credentials: embedded C2PA first, then TrustMark fallback.
//!
//! Entry points: `read_credentials_from_file` and `read_credentials_from_url`.

use std::io::Write;
use std::path::Path;
use std::sync::OnceLock;
use std::time::Duration;

use crate::trustmark;
use crate::types::{C2PAReadError, C2PAResult};

/// Shared HTTP client for URL fetches (connection reuse, one-time init).
static HTTP_CLIENT: OnceLock<reqwest::Client> = OnceLock::new();

fn http_client() -> &'static reqwest::Client {
    HTTP_CLIENT.get_or_init(|| {
        reqwest::Client::builder()
            .timeout(Duration::from_secs(60))
            .build()
            .expect("reqwest client")
    })
}

// -----------------------------------------------------------------------------
// File
// -----------------------------------------------------------------------------

/// Read from a local file: try embedded C2PA, then TrustMark, then return "none".
pub fn read_credentials_from_file(path: impl AsRef<Path>) -> Result<C2PAResult, C2PAReadError> {
    let path = path.as_ref();

    if let Ok(reader) = c2pa::Reader::from_file(path) {
        return Ok(C2PAResult::embedded(reader.detailed_json()));
    }

    if let Some(trust_mark_data) = trustmark::run_decoder(path) {
        return Ok(C2PAResult::trust_mark(trust_mark_data));
    }

    Ok(C2PAResult::none())
}

// -----------------------------------------------------------------------------
// URL
// -----------------------------------------------------------------------------

/// Read from a URL: download to a temp file, then read as file.
pub async fn read_credentials_from_url(url: &str) -> Result<C2PAResult, C2PAReadError> {
    let url = url.trim();
    if url.is_empty() || (!url.starts_with("http://") && !url.starts_with("https://")) {
        return Err(C2PAReadError::InvalidInput);
    }

    let bytes = http_client().get(url).send().await?.bytes().await?;

    let ext = url
        .split('?')
        .next()
        .and_then(|s| s.rsplit('.').next())
        .unwrap_or("bin");
    let mut temp = tempfile::Builder::new()
        .suffix(&format!(".{ext}"))
        .tempfile()?;
    std::io::Write::write_all(&mut temp, &bytes)?;
    temp.flush()?;

    let result = read_credentials_from_file(temp.path());
    temp.close().ok();
    result
}

// -----------------------------------------------------------------------------
// Tests
// -----------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn read_credentials_from_file_no_credentials_returns_success() {
        let temp = tempfile::NamedTempFile::new().unwrap();
        let result = read_credentials_from_file(temp.path()).unwrap();
        assert!(result.success);
        assert!(!result.has_credentials);
        assert!(result.manifest_data.is_none());
        assert!(result.trust_mark_data.is_none());
    }
}
