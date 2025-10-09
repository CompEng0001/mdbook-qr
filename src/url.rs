use anyhow::{anyhow, Result};
use mdbook::preprocess::PreprocessorContext;
use std::env;

/// Resolve URL from config, then [output.html].site-url, then GitHub Pages.
pub fn resolve_url(ctx: &PreprocessorContext, configured: Option<&str>) -> Result<String> {
    if let Some(u) = configured {
        return Ok(u.to_string());
    }
    if let Some(url) = ctx
        .config.get("output.html")
        .and_then(|h| h.get("site-url"))
        .and_then(|v| v.as_str())
    {
        return Ok(url.to_string());
    }
    if let Ok(repo) = env::var("GITHUB_REPOSITORY") {
        if let Some((owner, repo_name)) = repo.split_once('/') {
            return Ok(format!("https://{}.github.io/{}", owner, repo_name));
        }
    }
    Err(anyhow!("No URL found. Set [preprocessor.qr].url or [output.html].site-url."))
}
