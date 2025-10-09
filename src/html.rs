//! HTML utilities for injecting QR `<img>` tags into chapter content.
//!
//! This module provides helpers that walk an mdBook [`Book`] and replace
//! a given marker (e.g. `"{{QR_CODE}}"`) with an `<img>` whose `src` is
//! computed **relative to each chapter** so links remain valid regardless
//! of the chapter's directory depth.
//!
//! Path handling notes:
//! - The relative path is computed with [`pathdiff::diff_paths`] from each
//!   chapter directory to `qr_rel_under_src` (which is a path *under* the
//!   book's `src` directory).
//! - On Windows, backslashes are normalized to forward slashes for HTML.
//! - If the relative path has no `/`, a `./` prefix is added for robustness.
//! - If the path begins with `/`, the leading slash is trimmed to avoid
//!   absolute paths in the generated HTML.

use anyhow::Result;
use mdbook::book::{Book, BookItem};
use pathdiff::diff_paths;
use std::path::{Path, PathBuf};

/// Replace **all** occurrences of `marker` in every chapter with an `<img>` tag,
/// computing a chapter-relative `src` that points to `qr_rel_under_src`.
///
/// The generated tag looks like:
///
/// ```html
/// <img src="./path/to/qr.png" alt="QR code" style="height:128px;width:128px" loading="eager">
/// ```
///
/// Width/height styles are only included when `fit_h` / `fit_w` are greater than `0`.
/// Paths are normalized to forward slashes for portability.
///
/// # Parameters
///
/// - `book`: The mutable mdBook [`Book`] to transform.
/// - `marker`: The placeholder to replace (e.g., `"{{QR_CODE}}"` or a custom marker).
/// - `src_dir`: Absolute or canonical path to the book’s `src` directory (chapter paths are relative to this).
/// - `qr_rel_under_src`: The QR image path **under** `src_dir` (e.g., `"qr/mdbook-qr-code.png"`).
/// - `fit_h`: Height in pixels for the `<img>`; `0` to omit.
/// - `fit_w`: Width  in pixels for the `<img>`; `0` to omit.
///
/// # Returns
///
/// `Ok(())` on success. This function currently doesn’t produce errors but returns
/// `anyhow::Result` for consistency with other I/O-bound helpers and future expansion.
///
/// # Behavior and details
///
/// - Only `BookItem::Chapter` items are modified; other items are left untouched.
/// - If a chapter has no path, `qr_rel_under_src` is used as given (normalized).
/// - The replacement is a plain string `replace`, so **all** occurrences of `marker`
///   in a chapter’s content are replaced.
/// - Adds `loading="eager"` to avoid lazy-loading latency for QR codes that are intended
///   to be immediately visible.
///
/// # Examples
///
/// ```no_run
/// use mdbook::book::{Book, Chapter};
/// use std::path::Path;
///
/// // Suppose your book has a chapter at `src/intro.md`:
/// let mut book = Book::new();
/// let mut ch = Chapter::new("Intro", "Content with {{QR_CODE}} marker".into(), "intro.md".into());
/// book.sections.push(mdbook::book::BookItem::Chapter(ch));
///
/// // Compute paths (normally taken from mdBook context / config):
/// let src_dir = Path::new("/abs/path/to/book/src");
/// let qr_rel = Path::new("qr/mdbook-qr-code.png"); // under `src/`
///
/// mdbook_qr::html::inject_marker_relative(
///     &mut book,
///     "{{QR_CODE}}",
///     src_dir,
///     qr_rel,
///     128, // height px
///     128, // width px
/// ).unwrap();
/// ```
pub fn inject_marker_relative(
    book: &mut Book,
    marker: &str,
    src_dir: &Path,
    qr_rel_under_src: &Path,
    fit_h: u32,
    fit_w: u32,
) -> Result<()> {
    for section in book.sections.iter_mut() {
        if let BookItem::Chapter(ch) = section {
            if !ch.content.contains(marker) {
                continue;
            }

            if let Some(ch_rel_path) = &ch.path {
                // Determine the chapter directory relative to src/
                let ch_abs = src_dir.join(ch_rel_path);
                let ch_dir: PathBuf = ch_abs
                    .parent()
                    .map(|p| p.to_path_buf())
                    .unwrap_or_else(|| src_dir.to_path_buf());

                // Compute chapter-relative path to the QR image
                let rel = diff_paths(qr_rel_under_src, &ch_dir)
                    .unwrap_or_else(|| qr_rel_under_src.to_path_buf());

                // Normalize for HTML
                let mut rel_str = rel.to_string_lossy().replace('\\', "/");
                if !rel_str.contains('/') && !rel_str.starts_with("./") {
                    rel_str = format!("./{}", rel_str);
                } else if rel_str.starts_with('/') {
                    rel_str = rel_str.trim_start_matches('/').to_string();
                }

                // Build optional style attribute
                let mut style = String::new();
                let mut items: Vec<String> = Vec::new();
                if fit_h > 0 {
                    items.push(format!("height:{}px", fit_h));
                }
                if fit_w > 0 {
                    items.push(format!("width:{}px", fit_w));
                }
                if !items.is_empty() {
                    style = format!(r#" style="{}""#, items.join(";"));
                }

                let img = format!(
                    r#"<img src="{rel}" alt="QR code"{style} loading="eager">"#,
                    rel = rel_str,
                    style = style
                );

                ch.content = ch.content.replace(marker, &img);
            }
        }
    }
    Ok(())
}
