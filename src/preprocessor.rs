use anyhow::Result;
use mdbook::book::Book;
use mdbook::errors::Error;
use mdbook::preprocess::{CmdPreprocessor, Preprocessor, PreprocessorContext};
use std::io;

use crate::config::QrConfig;

pub struct QrPreprocessor;
impl QrPreprocessor { pub fn new() -> Self { Self } }

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
        eprintln!(
            "Warning: The '{}' plugin was built against {}, called from {}",
            pre.name(), mdbook::MDBOOK_VERSION, ctx.mdbook_version
        );
    }
    pre.run(&ctx, book).map(|processed| {
        serde_json::to_writer(io::stdout(), &processed).expect("write preprocessor output");
    }).map_err(|e| anyhow::anyhow!(e))
}

fn run_impl(ctx: &PreprocessorContext, book: &mut Book) -> Result<()> {
    let cfg: QrConfig = config_from_ctx(ctx).unwrap_or_default();
    if !cfg.is_enabled() { return Ok(()); }

    let src_dir = ctx.config.book.src.clone();

    // 1) Warn once for invalid customs (marker missing)
    cfg.warn_invalid_customs();

    // 2) Build valid profiles once (pure, no warnings)
    let profiles = cfg.profiles();

    // 3) Warn once if any duplicate marker among valid profiles
    if let Some(dupe) = QrConfig::duplicate_marker_from(profiles.iter()) {
        eprintln!("[mdbook-qr] Warning: duplicate marker configured: {dupe}");
    }

    // 4) Track image path collisions (warn only)
    use std::collections::HashMap;
    let mut path_to_marker: HashMap<std::path::PathBuf, String> = HashMap::new();

    for profile in profiles.into_iter().filter(|p| p.is_enabled()) {
        let marker = profile.marker.as_ref().expect("valid profiles always have a marker");

        // If both url and qr-path are missing, skip image creation (warn here or leave prior warning only)
        if profile.url.is_none() && profile.qr_path.is_none() {
            eprintln!(
                "[mdbook-qr] Warning: profile for marker '{}' has neither `url` nor `qr-path`; \
                 skipping image generation.",
                marker
            );
            continue;
        }

        // Resolve URL (fallbacks inside). If still none, skip with warning.
        let url = match crate::url::resolve_url(ctx, profile.url.as_deref()) {
            Ok(u) => u,
            Err(_) => {
                eprintln!(
                    "[mdbook-qr] Warning: could not resolve URL for marker '{}'; skipping image.",
                    marker
                );
                continue;
            }
        };

        // Resolve output path (explicit per-profile qr-path or derived from marker)
        let qr_rel_under_src = crate::util::resolve_profile_path(
            &src_dir,
            profile.qr_path.as_deref(),
            marker,
        );

        if let Some(prev) = path_to_marker.insert(qr_rel_under_src.clone(), marker.clone()) {
            if prev != *marker {
                eprintln!(
                    "[mdbook-qr] Warning: image path collision: '{}' and '{}' both map to '{}'. \
                     The latter may overwrite the former.",
                    prev, marker, qr_rel_under_src.display()
                );
            }
        }

        let (fit_w, fit_h) = crate::util::pass_fit_dims(&profile.fit);
        let margin = profile.margin.unwrap_or(2);
        let shape  = profile.shape.to_shape();
        let bg     = profile.background_color();
        let fg     = profile.module_color();

        crate::image::write_qr_png(
            &url, &ctx.root, &qr_rel_under_src,
            fit_w, fit_h, margin,
            Some(shape), bg, fg,
        )?;

        crate::html::inject_marker_relative(
            book,
            marker,
            &src_dir,
            &qr_rel_under_src,
            fit_h,
            fit_w,
        )?;
    }

    Ok(())
}


fn config_from_ctx(ctx: &PreprocessorContext) -> Option<QrConfig> {
    ctx.config
        .get_preprocessor("qr")
        .and_then(|table| toml::from_str(&toml::to_string(table).ok()?).ok())
}
