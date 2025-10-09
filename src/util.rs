//! Internal utilities for path resolution, sizing, and marker normalization.
//!
//! This module provides small helpers shared by the QR preprocessor:
//!
//! - Safe dimension handling (`pass_fit_dims`, `clamp_nonzero`)
//! - Marker normalization (`slug_from_marker`)
//! - Output path derivation and resolution (`derived_default_path`, `resolve_profile_path`)

use crate::config::FitConfig;
use std::path::{Path, PathBuf};

/// Default image size (in pixels) used when no width or height is specified.
pub const DEFAULT_SIZE: u32 = 200;

/// Compute the effective width and height for a profile’s QR image.
///
/// The behavior mirrors mdBook’s convenience handling:
///
/// - If neither `fit.width` nor `fit.height` is given → both default to [`DEFAULT_SIZE`].
/// - If only one is given → the other mirrors it.
/// - Zero values are clamped to [`DEFAULT_SIZE`] with a warning.
///
/// # Parameters
///
/// - `fit`: The [`FitConfig`] structure from the profile.
///
/// # Returns
///
/// `(width, height)` tuple in pixels, guaranteed non-zero.
pub fn pass_fit_dims(fit: &FitConfig) -> (u32, u32) {
    match (fit.width, fit.height) {
        (None, None) => (DEFAULT_SIZE, DEFAULT_SIZE),
        (Some(w), None) => {
            let ww = clamp_nonzero("fit.width", w, DEFAULT_SIZE);
            (ww, ww)
        }
        (None, Some(h)) => {
            let hh = clamp_nonzero("fit.height", h, DEFAULT_SIZE);
            (hh, hh)
        }
        (Some(w), Some(h)) => {
            let ww = clamp_nonzero("fit.width", w, DEFAULT_SIZE);
            let hh = clamp_nonzero("fit.height", h, DEFAULT_SIZE);
            (ww, hh)
        }
    }
}

/// Clamp a numeric value to a non-zero fallback, logging a warning if needed.
///
/// # Parameters
///
/// - `_label`: Descriptive label for diagnostics (e.g., `"fit.width"`).
/// - `value`: The provided numeric value.
/// - `fallback`: The replacement to use if `value` is `0`.
///
/// # Returns
///
/// Either `value` if non-zero, or `fallback` otherwise.
///
/// # Example
///
/// ```
/// assert_eq!(mdbook_qr::util::clamp_nonzero("fit.width", 0, 200), 200);
/// assert_eq!(mdbook_qr::util::clamp_nonzero("fit.height", 128, 200), 128);
/// ```
#[inline]
pub fn clamp_nonzero(_label: &str, value: u32, fallback: u32) -> u32 {
    if value == 0 {
        eprintln!("[mdbook-qr] Warning: value 0 is invalid; using {fallback}px");
        fallback
    } else {
        value
    }
}

/// Generate a filesystem-friendly slug from a marker such as `{{QR-FLYER}}`.
///
/// The slug is lower-cased, all non-alphanumeric characters are replaced with
/// underscores, and multiple underscores are collapsed.
///
/// ```
/// use mdbook_qr::util::slug_from_marker;
///
/// assert_eq!(slug_from_marker("{{QR-FLYER}}"), "qr_flyer");
/// assert_eq!(slug_from_marker("{{QR.CODE:Slide}}"), "qr_code_slide");
/// ```
pub fn slug_from_marker(marker: &str) -> String {
    let mut s = marker
        .trim()
        .trim_matches('{')
        .trim_matches('}')
        .to_string();
    s = s.chars().filter(|c| *c != '{' && *c != '}').collect();

    let mut out = String::with_capacity(s.len());
    for ch in s.chars() {
        if ch.is_ascii_alphanumeric() {
            out.push(ch.to_ascii_lowercase());
        } else {
            out.push('_');
        }
    }
    while out.contains("__") {
        out = out.replace("__", "_");
    }
    out.trim_matches('_').to_string()
}

/// Derive a default QR image path for a given marker.
///
/// Resulting path:
///
/// ```text
/// <src_dir>/qr/<slug>.png
/// ```
///
/// where `<slug>` is produced by [`slug_from_marker`].
///
/// # Example
///
/// ```
/// use std::path::Path;
/// use mdbook_qr::util::derived_default_path;
///
/// let p = derived_default_path(Path::new("book/src"), "{{QR-MAIN}}");
/// assert!(p.ends_with("qr/qr_main.png"));
/// ```
pub fn derived_default_path(src_dir: &Path, marker: &str) -> PathBuf {
    src_dir
        .join("qr")
        .join(format!("{}.png", slug_from_marker(marker)))
}

/// Resolve the final image path for a given profile.
///
/// Resolution rules:
///
/// 1. If `qr_path` is provided:
///    - Absolute → returned as-is.
///    - Relative → joined under `src_dir`.
/// 2. Otherwise, fall back to [`derived_default_path`].
///
/// # Parameters
///
/// - `src_dir`: The mdBook source directory.
/// - `qr_path`: Optional path from configuration.
/// - `marker`: Marker string used for default derivation.
///
/// # Returns
///
/// Absolute or relative [`PathBuf`] resolved according to the above rules.
///
/// # Example
///
/// ```
/// use std::path::Path;
/// use mdbook_qr::util::resolve_profile_path;
///
/// let src = Path::new("book/src");
/// let custom = Some("custom/qr.png");
///
/// let p1 = resolve_profile_path(src, custom, "{{QR-A}}");
/// let p2 = resolve_profile_path(src, None, "{{QR-B}}");
///
/// assert!(p1.ends_with("book/src/custom/qr.png"));
/// assert!(p2.ends_with("qr/qr_b.png"));
/// ```
pub fn resolve_profile_path(src_dir: &Path, qr_path: Option<&str>, marker: &str) -> PathBuf {
    if let Some(p) = qr_path {
        let pb = PathBuf::from(p);
        if pb.is_absolute() {
            pb
        } else {
            src_dir.join(pb)
        }
    } else {
        derived_default_path(src_dir, marker)
    }
}
