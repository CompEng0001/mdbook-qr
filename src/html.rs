use mdbook::book::{Book, BookItem};
use pathdiff::diff_paths;
use std::path::{Path, PathBuf};


/// Replace `marker` with `replacement` in `content`, but:
/// - Do NOT replace inside fenced code blocks (``` or ~~~).
/// - Still allow replacement inside `~~~admonish ... ~~~` blocks (treated as normal text).
/// - Do NOT replace inside inline code spans enclosed by backticks (`...` or ```` ... ````).
fn replace_markers_outside_code(content: &str, marker: &str, replacement: &str) -> String {
    let mut out = String::with_capacity(content.len());

    // Fence tracking
    let mut in_fence = false;
    let mut fence_char = '\0';
    let mut fence_len: usize = 0;

    // Utility: detect a fence line and return (is_fence, fence_char, fence_len, info_string)
    fn parse_fence(line: &str) -> Option<(char, usize, &str)> {
        // Allow up to 3 leading spaces per CommonMark
        let trimmed_lead = line.strip_prefix("   ")
            .or_else(|| line.strip_prefix("  "))
            .or_else(|| line.strip_prefix(" "))
            .unwrap_or(line);

        let bytes = trimmed_lead.as_bytes();
        if bytes.is_empty() { return None; }

        let first = bytes[0] as char;
        if first != '`' && first != '~' {
            return None;
        }

        // Count run length of the same char
        let mut i = 0;
        while i < bytes.len() && (bytes[i] as char) == first {
            i += 1;
        }
        if i < 3 {
            return None;
        }

        // Info string after the fence run (can be empty)
        let info = &trimmed_lead[i..];
        Some((first, i, info))
    }

    // Replace marker in a *single line* but skip inline code spans marked by backticks.
    fn replace_outside_inline_code(line: &str, marker: &str, repl: &str) -> String {
        let mut result = String::with_capacity(line.len());
        let mut i = 0;
        let line_bytes = line.as_bytes();

        // Tracks active inline code span delimited by N backticks
        let mut inline_bt_count: Option<usize> = None;

        while i < line.len() {
            // SAFETY: i is always maintained at a char boundary
            let ch = line[i..].chars().next().unwrap();
            let ch_len = ch.len_utf8();

            if ch == '`' {
                // Count a run of backticks. Backticks are ASCII => 1 byte each.
                let mut j = i + ch_len; // i + 1
                let mut count = 1;
                while j < line.len() && line_bytes[j] == b'`' {
                    j += 1;
                    count += 1;
                }

                // Copy the whole backtick run verbatim
                result.push_str(&line[i..j]);

                match inline_bt_count {
                    None => inline_bt_count = Some(count),            // open span
                    Some(open) if open == count => inline_bt_count = None, // close span
                    _ => { /* mismatched counts → treat as raw */ }
                }

                i = j;
                continue;
            }

            // If not inside inline code, we can attempt marker replacement
            if inline_bt_count.is_none() && line[i..].starts_with(marker) {
                result.push_str(repl);
                i += marker.len(); // marker is ASCII, so byte-len is safe
                continue;
            }

            // Default: copy this character as-is
            result.push(ch);
            i += ch_len;
        }

        result
    }


    for line in content.split_inclusive('\n') {
        // We operate per physical line (including its trailing '\n')
        // Use a copy without the trailing '\n' to parse fences cleanly
        let (line_body, line_suffix_nl) = if line.ends_with('\n') {
            (&line[..line.len() - 1], "\n")
        } else {
            (line, "")
        };

        // Check for a fence delimiter
        if let Some((ch, run_len, info)) = parse_fence(line_body) {
            // Is this a *closing* fence for the current one?
            if in_fence {
                if ch == fence_char && run_len >= fence_len {
                    // Close the fence
                    in_fence = false;
                    fence_char = '\0';
                    fence_len = 0;
                }
                // Copy the delimiter line as-is
                out.push_str(line_body);
                out.push_str(line_suffix_nl);
                continue;
            } else {
                // Opening fence. Treat '~~~admonish ...' as NON-code fence:
                let info_lower = info.trim().to_ascii_lowercase();
                let is_admonish = info_lower.starts_with("admonish");

                if !is_admonish {
                    // Enter code fence
                    in_fence = true;
                    fence_char = ch;
                    fence_len = run_len;
                    out.push_str(line_body);
                    out.push_str(line_suffix_nl);
                    continue;
                }
                // For admonish, fall through (not a code fence): we still replace markers inside.
            }
        }

        if in_fence {
            // Inside a code fence → no replacement
            out.push_str(line_body);
            out.push_str(line_suffix_nl);
        } else {
            // Outside code fences → replace markers, but skip inline code spans
            let replaced = replace_outside_inline_code(line_body, marker, replacement);
            out.push_str(&replaced);
            out.push_str(line_suffix_nl);
        }
    }

    out
}

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
                ch.content = replace_markers_outside_code(&ch.content, marker, &img);
            }
        }
    }
    Ok(())
}
