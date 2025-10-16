use anyhow::Result;
use log::{warn, info, debug};
use mdbook::book::{Book, BookItem};
use mdbook::errors::Error;
use mdbook::preprocess::{CmdPreprocessor, Preprocessor, PreprocessorContext};
use std::collections::HashMap;
use std::io;

use crate::config::{FailureMode, QrConfig, Profile, FitConfig, ShapeFlags, ColorCfg};

pub struct QrPreprocessor;
impl QrPreprocessor {
    pub fn new() -> Self { Self }
}

impl Preprocessor for QrPreprocessor {
    fn name(&self) -> &str { "qr" }
    fn run(&self, ctx: &PreprocessorContext, mut book: Book) -> std::result::Result<Book, Error> {
        run_impl(ctx, &mut book).map_err(Error::from)?;
        Ok(book)
    }
    fn supports_renderer(&self, _renderer: &str) -> bool { true }
}

pub fn run_preprocessor_once() -> Result<()> {
    let pre = QrPreprocessor::new();
    let (ctx, book) = CmdPreprocessor::parse_input(io::stdin())?;

    if ctx.mdbook_version != mdbook::MDBOOK_VERSION {
        warn!(
            "The '{}' plugin was built against {}, called from {}",
            pre.name(), mdbook::MDBOOK_VERSION, ctx.mdbook_version
        );
    }

    pre.run(&ctx, book)
        .map(|processed| {
            serde_json::to_writer(io::stdout(), &processed)
                .expect("write preprocessor output");
        })
        .map_err(|e| anyhow::anyhow!(e))
}

/// Does the given marker appear in any chapter?
fn marker_in_book(book: &Book, marker: &str) -> bool {
    book.sections.iter().any(|item| {
        if let BookItem::Chapter(ch) = item {
            ch.content.contains(marker)
        } else { false }
    })
}

/// Build a `Profile` from the bare `[preprocessor.qr.custom]` table (no marker).
/// Avoids `toml` type/version clashes by reading primitives only.
fn load_custom_defaults(ctx: &PreprocessorContext) -> Option<Profile> {
    let custom = ctx
        .config
        .get("preprocessor")?
        .get("qr")?
        .get("custom")?
        .as_table()?;

    let mut p = Profile {
        marker: None,
        enable: None,
        url: None,
        qr_path: None,
        fit: FitConfig::default(),
        margin: None,
        shape: ShapeFlags::default(),
        background: None,
        module: None,
    };

    if let Some(v) = custom.get("enable").and_then(|v| v.as_bool()) {
        p.enable = Some(v);
    }
    if let Some(v) = custom.get("url").and_then(|v| v.as_str()) {
        p.url = Some(v.to_string());
    }
    if let Some(v) = custom.get("qr-path").and_then(|v| v.as_str()) {
        p.qr_path = Some(v.to_string());
    }
    if let Some(v) = custom.get("margin").and_then(|v| v.as_integer()) {
        if v >= 0 { p.margin = Some(v as u32); }
    }

    if let Some(fit_tbl) = custom.get("fit").and_then(|v| v.as_table()) {
        if let Some(w) = fit_tbl.get("width").and_then(|v| v.as_integer()) {
            if w >= 0 { p.fit.width = Some(w as u32); }
        }
        if let Some(h) = fit_tbl.get("height").and_then(|v| v.as_integer()) {
            if h >= 0 { p.fit.height = Some(h as u32); }
        }
    }

    if let Some(shape_tbl) = custom.get("shape").and_then(|v| v.as_table()) {
        p.shape.square          = shape_tbl.get("square").and_then(|v| v.as_bool()).unwrap_or(false);
        p.shape.circle          = shape_tbl.get("circle").and_then(|v| v.as_bool()).unwrap_or(false);
        p.shape.rounded_square  = shape_tbl.get("rounded_square").and_then(|v| v.as_bool()).unwrap_or(false);
        p.shape.vertical        = shape_tbl.get("vertical").and_then(|v| v.as_bool()).unwrap_or(false);
        p.shape.horizontal      = shape_tbl.get("horizontal").and_then(|v| v.as_bool()).unwrap_or(false);
        p.shape.diamond         = shape_tbl.get("diamond").and_then(|v| v.as_bool()).unwrap_or(false);
    }

    if let Some(bg) = custom.get("background").and_then(|v| v.as_str()) {
        p.background = Some(ColorCfg::Hex(bg.to_string()));
    }
    if let Some(fg) = custom.get("module").and_then(|v| v.as_str()) {
        p.module = Some(ColorCfg::Hex(fg.to_string()));
    }

    Some(p)
}

fn run_impl(ctx: &PreprocessorContext, book: &mut Book) -> Result<()> {
    let cfg: QrConfig = config_from_ctx(ctx).unwrap_or_default();
    if !cfg.is_enabled() { return Ok(()); }
    let on_failure = cfg.on_failure.clone();
    let src_dir = ctx.config.book.src.clone();

    cfg.warn_invalid_customs();

    // 1) Detect a *bare* [preprocessor.qr.custom] table (no named subtables)
    let has_bare_custom = ctx
        .config
        .get("preprocessor")
        .and_then(|pp| pp.get("qr"))
        .and_then(|qr| qr.get("custom"))
        .map(|v| v.is_table())
        .unwrap_or(false)
        && cfg.custom.is_empty();

    // 2) Load defaults from bare custom (for inheritance only; never generates by itself)
    let custom_defaults = load_custom_defaults(ctx);

    // 3) Build profiles
    let mut profiles: Vec<Profile> = Vec::new();
    let default_p = cfg.default_profile();

    // ── CHANGED: only include the default if there is NOT a bare custom table
    if !has_bare_custom {
        profiles.push(default_p.clone());
    } else {
        warn!(
            "mdbook-qr: bare [preprocessor.qr.custom] present with no named subtables; \
             suppressing default '{{QR_CODE}}' until a named custom (e.g., [preprocessor.qr.custom.flyer]) exists."
        );
    }

    // Named customs (must have marker)
    for (_name, child) in &cfg.custom {
        if child.marker.is_none() {
            warn!("mdbook-qr: custom entry missing `marker`; skipping.");
            continue;
        }

        // default -> child
        let mut eff = QrConfig::inherit(&default_p, child);

        // overlay bare [preprocessor.qr.custom] defaults if present
        if let Some(cd) = &custom_defaults {
            eff = QrConfig::inherit(cd, &eff);
            // child's explicit fields win back
            eff.marker = child.marker.clone();
            if child.qr_path.is_some() {
                eff.qr_path = child.qr_path.clone();
            }
        }
        profiles.push(eff);
    }

    for p in &profiles {
        if let Some(m) = &p.marker {
            info!("mdbook-qr: profile queued -> marker {}", m);
        }   
    }

    // Optional: warn on duplicate markers
    if let Some(dupe) = QrConfig::duplicate_marker_from(profiles.iter()) {
        warn!("duplicate marker configured: {dupe}");
    }

    // Track file-path collisions (warn only)
    let mut path_to_marker: HashMap<std::path::PathBuf, String> = HashMap::new();

    for profile in profiles.into_iter().filter(|p| p.is_enabled()) {
        let marker = profile.marker.as_ref().expect("profiles here always have marker");

        // Only generate if the marker is used
        if !marker_in_book(book, marker) {
            debug!("mdbook-qr: marker '{}' not found in any chapter; skipping", marker);
            continue;
        }

        // Resolve URL (explicit -> env fallback)
        let url = match crate::url::resolve_url(profile.url.as_deref()) {
            Ok(u) => u,
            Err(_) => match on_failure {
                FailureMode::Continue => {
                    warn!(
                        "could not resolve URL for '{}'; set `preprocessor.qr.url` \
                         or export GITHUB_REPOSITORY; skipping image.",
                        marker
                    );
                    continue;
                }
                FailureMode::Bail => {
                    anyhow::bail!(
                        "mdbook-qr: could not resolve URL for '{}'; \
                         set `preprocessor.qr.url` or export GITHUB_REPOSITORY.",
                        marker
                    );
                }
            },
        };

        // Relative output path (under book src)
        let qr_rel_under_src = crate::util::resolve_profile_path(
            &src_dir, profile.qr_path.as_deref(), marker,
        );

        // Warn on two markers mapping to same file
        if let Some(prev) = path_to_marker.insert(qr_rel_under_src.clone(), marker.clone()) {
            if prev != *marker {
                warn!(
                    "image path collision: '{}' and '{}' both map to '{}'. \
                     The latter may overwrite the former.",
                    prev, marker, qr_rel_under_src.display()
                );
            }
        }

        // Safety guard: if writing to derived default file and it exists, require explicit qr-path
        let derived_default = crate::util::derived_default_path(&src_dir, "{{QR_CODE}}");
        if qr_rel_under_src == derived_default && profile.qr_path.is_none() {
            let abs_candidate = ctx.root.join(&qr_rel_under_src);
            if abs_candidate.exists() {
                warn!(
                    "mdbook-qr: '{}' already exists; refusing to overwrite derived default. \
                     Set an explicit `qr-path` for marker {} to proceed.",
                    abs_candidate.display(), marker
                );
                continue;
            }
        }

        // Render + inject
        let (fit_w, fit_h) = crate::util::pass_fit_dims(&profile.fit);
        let margin = profile.margin.unwrap_or(2);
        let shape  = profile.shape.to_shape();
        let bg     = profile.background_color();
        let fg     = profile.module_color();

        let (_abs_out, content_hash) = crate::image::write_qr_png(
            &url, &ctx.root, &qr_rel_under_src, fit_w, fit_h, margin, Some(shape), bg, fg,
        )?;

        crate::html::inject_marker_relative(
            book, marker, &src_dir, &qr_rel_under_src, fit_h, fit_w, Some(&content_hash),
        )?;
    }

    Ok(())
}

/// Deserialize [preprocessor.qr] from the mdBook context.
fn config_from_ctx(ctx: &PreprocessorContext) -> Option<QrConfig> {
    ctx.config
        .get_preprocessor("qr")
        .and_then(|table| toml::from_str(&toml::to_string(table).ok()?).ok())
}
