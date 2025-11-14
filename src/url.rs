use anyhow::{anyhow, Result};
use log::{debug, warn};
use std::env;

fn is_abs_http(u: &str) -> bool {
    let lu = u.trim().to_lowercase();
    lu.starts_with("http://") || lu.starts_with("https://")
}

/// Resolve URL (site-url intentionally ignored):
/// 1) explicit profile url (preprocessor.qr.url or custom profile url)
/// 2) CI fallback from GITHUB_REPOSITORY -> https://{owner}.github.io/{repo}
/// 3) localhost-qr flag -> http://127.0.0.1:3000/
pub fn resolve_url(url: Option<&str>, localhost_qr: bool) -> Result<String> {
    // 1) explicit preprocessor url wins
    if let Some(u) = url {
        if !is_abs_http(u) {
            warn!(
                "preprocessor.qr.url is not absolute ('{}'); QR will encode it as-is",
                u
            );
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

    if localhost_qr {
        let u = "http://127.0.0.1:3000/".to_string();
        debug!("using localhost-qr fallback = {}", u);
        return Ok(u);
    }
    Err(anyhow!("no URL configured and no viable fallback"))
}
