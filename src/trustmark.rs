//! TrustMark watermark detection using the [Rust TrustMark crate](https://docs.rs/trustmark/latest/trustmark/).
//!
//! When embedded C2PA is not present, we decode pixel-based watermarks via the
//! CAI TrustMark implementation. The decoder is lazily initialized and cached
//! so ONNX models are loaded once per process. Set `TRUSTMARK_MODELS` to the
//! path of the models directory or place them in `./models`. See
//! <https://opensource.contentauthenticity.org/docs/trustmark/rust/>.

use std::path::{Path, PathBuf};
use std::sync::OnceLock;

use image::DynamicImage;
use trustmark::{Trustmark, Variant, Version};

use crate::types::TrustMarkWatermarkData;

/// Default variant for decoding (matches common production use; same as Python 'P').
const DECODE_VARIANT: Variant = Variant::P;

/// Cached decoder: initialized once on first use to avoid reloading ONNX models per request.
static DECODER: OnceLock<Option<Trustmark>> = OnceLock::new();

// -----------------------------------------------------------------------------
// Model resolution
// -----------------------------------------------------------------------------

/// Resolve the directory containing TrustMark ONNX models.
fn find_models_dir() -> Option<PathBuf> {
    if let Ok(p) = std::env::var("TRUSTMARK_MODELS") {
        let path = PathBuf::from(p);
        if path.is_dir() {
            return Some(path);
        }
    }
    for c in ["models", "../models"] {
        let p = PathBuf::from(c);
        if p.is_dir() {
            return Some(p);
        }
    }
    if let Ok(exe) = std::env::current_exe() {
        if let Some(dir) = exe.parent() {
            let models = dir.join("../models");
            if models.is_dir() {
                return Some(models);
            }
        }
    }
    None
}

/// Returns the cached Trustmark decoder, or initializes it from the first valid models dir.
fn get_decoder() -> Option<&'static Trustmark> {
    DECODER
        .get_or_init(|| {
            find_models_dir()
                .and_then(|path| Trustmark::new(path, DECODE_VARIANT, Version::default()).ok())
        })
        .as_ref()
}

// -----------------------------------------------------------------------------
// Schema and result mapping
// -----------------------------------------------------------------------------

fn version_to_schema(v: Version) -> &'static str {
    match v {
        Version::BchSuper => "BCH_SUPER",
        Version::Bch5 => "BCH_5",
        Version::Bch4 => "BCH_4",
        Version::Bch3 => "BCH_3",
    }
}

fn decoded_payload_to_result(decoded: &str, version: Version) -> TrustMarkWatermarkData {
    let decoded = decoded.trim();
    let manifest_url = if decoded.starts_with("http://") || decoded.starts_with("https://") {
        Some(decoded.to_string())
    } else {
        None
    };
    TrustMarkWatermarkData {
        identifier: decoded.to_string(),
        schema: version_to_schema(version).to_string(),
        raw: decoded.to_string(),
        manifest_url,
    }
}

// -----------------------------------------------------------------------------
// Public API
// -----------------------------------------------------------------------------

/// Run the TrustMark decoder on an image path; returns watermark data if found.
/// Uses a cached decoder so model loading happens at most once per process.
pub(crate) fn run_decoder(path: &Path) -> Option<TrustMarkWatermarkData> {
    let tm = get_decoder()?;
    let img: DynamicImage = image::open(path).ok()?;
    let decoded = tm.decode(img).ok()?;
    if decoded.trim().is_empty() {
        return None;
    }
    Some(decoded_payload_to_result(&decoded, Version::default()))
}
