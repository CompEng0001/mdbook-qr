use anyhow::{anyhow, Context, Result};
use fast_qr::convert::{image::ImageBuilder, Builder, Color, Shape};
use fast_qr::qr::QRBuilder;
use std::{fs, io::Write, path::{Path, PathBuf}};

pub fn write_qr_png(
    url: &str,
    root: &Path,
    qr_rel: &Path,
    fit_w: u32,
    fit_h: u32,
    margin: u32,
    shape: Option<Shape>,
    background: Option<Color>,
    module: Option<Color>,
) -> Result<(PathBuf, String)> {
    let qrcode = QRBuilder::new(url)
        .build()
        .map_err(|e| anyhow!("QR build error: {e:?}"))?;

    let mut out = root.join(qr_rel);
    if out.extension().and_then(|s| s.to_str()).unwrap_or("").to_ascii_lowercase() != "png" {
        out.set_extension("png");
    }
    if let Some(parent) = out.parent() {
        fs::create_dir_all(parent).with_context(|| format!("Creating {}", parent.display()))?;
    }

    let mut builder = ImageBuilder::default();
    builder.margin(margin as usize).fit_width(fit_w).fit_height(fit_h);
    if let Some(s) = shape      { builder.shape(s); }
    if let Some(bg) = background { builder.background_color(bg); }
    if let Some(fg) = module     { builder.module_color(fg); }

    let bytes = builder
        .to_bytes(&qrcode)
        .map_err(|e| anyhow!("PNG encode: {e}"))?;

    let _changed = write_if_changed(&out, &bytes)?;
    let hash = blake3::hash(&bytes).to_hex()[..12].to_string();
    Ok((out, hash))
}

fn write_if_changed(path: &Path, bytes: &[u8]) -> Result<bool> {
    if let Ok(existing) = fs::read(path) {
        if existing == bytes {
            return Ok(false);
        }
    }
    let parent = path.parent().unwrap_or_else(|| Path::new("."));
    let tmp = parent.join(format!(
        ".{}.tmp",
        path.file_name().and_then(|s| s.to_str()).unwrap_or("qr-image")
    ));
    {
        let mut f = fs::File::create(&tmp).with_context(|| format!("Creating {}", tmp.display()))?;
        f.write_all(bytes).with_context(|| format!("Writing {}", tmp.display()))?;
        let _ = f.sync_all();
    }
    fs::rename(&tmp, path).with_context(|| {
        format!("Renaming {} → {}", tmp.display(), path.display())
    })?;
    Ok(true)
}
