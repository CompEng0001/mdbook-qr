use anyhow::Result;
use mdbook::book::{Book, BookItem};
use pathdiff::diff_paths;
use std::path::{Path, PathBuf};

/// Replace all occurrences of `marker` with an <img> whose `src` is
/// chapter-relative to `qr_rel_under_src`.
pub fn inject_marker_relative(
    book: &mut Book,
    marker: &str,
    src_dir: &Path,
    qr_rel_under_src: &Path,
    fit_h: u32,
    fit_w: u32,
    cache_bust: Option<&str>,  // NEW
) -> anyhow::Result<()> {
    for section in book.sections.iter_mut() {
        if let BookItem::Chapter(ch) = section {
            if !ch.content.contains(marker) { continue; }

            if let Some(ch_rel_path) = &ch.path {
                let ch_abs = src_dir.join(ch_rel_path);
                let ch_dir: PathBuf = ch_abs
                    .parent()
                    .map(|p| p.to_path_buf())
                    .unwrap_or_else(|| src_dir.to_path_buf());

                let rel = diff_paths(qr_rel_under_src, &ch_dir)
                    .unwrap_or_else(|| qr_rel_under_src.to_path_buf());

                let mut rel_str = rel.to_string_lossy().replace('\\', "/");
                if !rel_str.contains('/') && !rel_str.starts_with("./") {
                    rel_str = format!("./{}", rel_str);
                } else if rel_str.starts_with('/') {
                    rel_str = rel_str.trim_start_matches('/').to_string();
                }

                if let Some(v) = cache_bust {
                    if rel_str.contains('?') { rel_str.push_str(&format!("&v={v}")); }
                    else { rel_str.push_str(&format!("?v={v}")); }
                }

                let mut style = String::new();
                let mut items: Vec<String> = Vec::new();
                if fit_h > 0 { items.push(format!("height:{}px", fit_h)); }
                if fit_w > 0 { items.push(format!("width:{}px", fit_w)); }
                if !items.is_empty() { style = format!(r#" style="{}""#, items.join(";")); }

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
