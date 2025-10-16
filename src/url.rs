// src/url.rs
use anyhow::{anyhow, Result};
use log::{debug, warn};
use mdbook::preprocess::PreprocessorContext;
use std::env;

fn is_abs_http(u: &str) -> bool {
    let lu = u.trim().to_lowercase();
    lu.starts_with("http://") || lu.starts_with("https://")
}

/// Resolve URL in this order (site-url intentionally ignored):
/// 1) explicit profile url (preprocessor.qr.url or custom profile url)
/// 2) CI fallback from GITHUB_REPOSITORY -> https://{owner}.github.io/{repo}
pub fn resolve_url(ctx: &PreprocessorContext, configured: Option<&str>) -> Result<String> {
    // 1) explicit preprocessor url wins
    if let Some(u) = configured {
        if !is_abs_http(u) {
            warn!("preprocessor.qr.url is not absolute ('{}'); QR will encode it as-is", u);
        }
        debug!("using explicit preprocessor.qr.url = {}", u);
        return Ok(u.to_string());
    }

    // 2) GitHub Pages fallback from CI
    if let Ok(repo) = env::var("GITHUB_REPOSITORY") {
        if let Some((owner, repo_name)) = repo.split_once('/') {
            let gh_pages = format!("https://{}.github.io/{}", owner, repo_name);
            debug!("using GITHUB_REPOSITORY fallback = {}", gh_pages);
            return Ok(gh_pages);
        }
    }

    // 3) nothing found
    Err(anyhow!(
        "could not resolve a URL: set `preprocessor.qr.url` or export GITHUB_REPOSITORY"
    ))
}
