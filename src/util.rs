use std::path::{Path, PathBuf};
use crate::config::FitConfig;

pub const DEFAULT_SIZE: u32 = 200;

pub fn pass_fit_dims(fit: &FitConfig) -> (u32, u32) {
    match (fit.width, fit.height) {
        (None, None) => (DEFAULT_SIZE, DEFAULT_SIZE),
        (Some(w), None) => { let ww = clamp_nonzero("fit.width", w, DEFAULT_SIZE); (ww, ww) }
        (None, Some(h)) => { let hh = clamp_nonzero("fit.height", h, DEFAULT_SIZE); (hh, hh) }
        (Some(w), Some(h)) => {
            let ww = clamp_nonzero("fit.width", w, DEFAULT_SIZE);
            let hh = clamp_nonzero("fit.height", h, DEFAULT_SIZE);
            (ww, hh)
        }
    }
}

#[inline]
pub fn clamp_nonzero(_label: &str, value: u32, fallback: u32) -> u32 {
    if value == 0 {
        eprintln!("[mdbook-qr] Warning: value 0 is invalid; using {fallback}px");
        fallback
    } else { value }
}

/// Slug from marker like "{{QR-FLYER}}" → "qr_flyer"
pub fn slug_from_marker(marker: &str) -> String {
    let mut s = marker.trim().trim_matches('{').trim_matches('}').to_string();
    s = s.chars().filter(|c| *c != '{' && *c != '}').collect();
    let mut out = String::with_capacity(s.len());
    for ch in s.chars() {
        if ch.is_ascii_alphanumeric() { out.push(ch.to_ascii_lowercase()); } else { out.push('_'); }
    }
    while out.contains("__") { out = out.replace("__", "_"); }
    out.trim_matches('_').to_string()
}

/// Default derived path: `<src_dir>/qr/<slug>.png`
pub fn derived_default_path(src_dir: &Path, marker: &str) -> PathBuf {
    src_dir.join("qr").join(format!("{}.png", slug_from_marker(marker)))
}

/// Resolve final profile path:
/// - If `qr_path` given: absolute → as-is; relative → join under `src_dir`
/// - Else: derive from marker under `<src_dir>/qr`
pub fn resolve_profile_path(src_dir: &Path, qr_path: Option<&str>, marker: &str) -> PathBuf {
    if let Some(p) = qr_path {
        let pb = PathBuf::from(p);
        if pb.is_absolute() { pb } else { src_dir.join(pb) }
    } else {
        derived_default_path(src_dir, marker)
    }
}
