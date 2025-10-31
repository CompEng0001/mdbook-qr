use std::path::{Path, PathBuf};
use anyhow::{Result, Context};
use std::fs;
use std::io::Write;
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

/// Fixed dev path when `localhost-qr = true`:
/// {book.src}/localhost/qr_localhost.png  (absolute, under repo root)
pub fn localhost_fixed_path(src_dir: &Path) -> PathBuf {
    src_dir.join("mdbook_qr").join("qr_localhost.png")
}

/// Ensure `.gitignore` has a glob ignoring:
///    /{book.src}/**/mdbook_qr/qr_localhost.png
/// Creates `.gitignore` if missing; idempotent.
pub fn ensure_gitignore_for_localhost(root: &Path, src_dir: &Path) -> Result<bool> {
    let gi_path = root.join(".gitignore");

    // Compute repo-relative src path
    let mut glob = format!("*{}/mdbook_qr/",
        src_dir.to_string_lossy().replace('\\', "/"));

    while glob.contains("//") {
        glob = glob.replace("//", "/");
    }

    let mut contents = fs::read_to_string(&gi_path).unwrap_or_default();
    if contents.lines().any(|l| l.trim() == glob) {
        return Ok(false);
    }
    if !contents.is_empty() && !contents.ends_with('\n') {
        contents.push('\n');
    }
    // Optional: tag for discoverability
    contents.push_str("# mdbook-qr (localhost image)\n");
    contents.push_str(&glob);
    contents.push('\n');

    let mut f = fs::OpenOptions::new()
        .create(true)
        .write(true)
        .truncate(true)
        .open(&gi_path)
        .with_context(|| format!("opening {}", gi_path.display()))?;
    f.write_all(contents.as_bytes())
        .with_context(|| format!("writing {}", gi_path.display()))?;
    Ok(true)
}

