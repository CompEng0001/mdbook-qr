//! Configuration structures for the **mdbook-qr** preprocessor.
//!
//! This module defines the configuration types used by `mdbook-qr`, including
//! per-marker profiles and shape/color handling.
//!
//! Each configuration type can be deserialized from `book.toml` via `serde`.

use fast_qr::convert::{Color, Shape};
use log::warn;
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, HashSet};

/// Flexible color input accepted in `book.toml`.
///
/// Supported forms:
///
/// - Hex strings: `"#000"`, `"#000000"`, `"#000000FF"`
/// - RGB arrays: `[0, 0, 0]`
/// - RGBA arrays: `[0, 0, 0, 255]`
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(untagged)]
pub enum ColorCfg {
    /// Hexadecimal color (e.g. `"#FF0000FF"`).
    Hex(String),
    /// RGBA array `[r,g,b,a]`.
    Rgba([u8; 4]),
    /// RGB array `[r,g,b]`.
    Rgb([u8; 3]),
}

impl ColorCfg {
    /// Convert the color configuration to a [`fast_qr::convert::Color`].
    ///
    /// This automatically interprets hex or numeric array formats.
    #[inline]
    pub fn to_color(&self) -> Color {
        match self {
            ColorCfg::Hex(s) => Color::from(s.as_str()),
            ColorCfg::Rgba(a4) => Color::from(*a4),
            ColorCfg::Rgb(a3) => Color::from(*a3),
        }
    }
}

/// Image fit configuration for the generated `<img>` tag.
#[derive(Clone, Debug, Deserialize, Serialize, Default)]
pub struct FitConfig {
    /// Target width of the QR code image (in pixels).
    pub width: Option<u32>,
    /// Target height of the QR code image (in pixels).
    pub height: Option<u32>,
}

/// Shape selection flags corresponding to [`fast_qr::convert::Shape`].
///
/// Only one should be `true`; if multiple are set, the first in precedence
/// order will be used (`circle → rounded_square → vertical → horizontal → diamond → square`).
#[derive(Clone, Debug, Deserialize, Serialize, Default)]
#[serde(default)]
pub struct ShapeFlags {
    pub square: bool,
    pub circle: bool,
    pub rounded_square: bool,
    pub vertical: bool,
    pub horizontal: bool,
    pub diamond: bool,
}

impl ShapeFlags {
    /// Return the selected [`Shape`] according to precedence.
    pub fn to_shape(&self) -> Shape {
        if self.circle {
            Shape::Circle
        } else if self.rounded_square {
            Shape::RoundedSquare
        } else if self.vertical {
            Shape::Vertical
        } else if self.horizontal {
            Shape::Horizontal
        } else if self.diamond {
            Shape::Diamond
        } else {
            Shape::Square
        }
    }

    /// Returns `true` if any shape flag is set.
    fn any_set(&self) -> bool {
        self.square
            || self.circle
            || self.rounded_square
            || self.vertical
            || self.horizontal
            || self.diamond
    }
}

/// Individual QR profile configuration.
///
/// Each custom marker entry inherits defaults from the main `[preprocessor.qr]` table
/// unless explicitly overridden.
#[derive(Clone, Debug, Deserialize, Serialize, Default)]
#[serde(rename_all = "kebab-case")]
pub struct Profile {
    /// Marker string, e.g. `"{{QR_FOOTER}}"`.
    pub marker: Option<String>,
    /// Output path for the PNG image.
    pub qr_path: Option<String>,
    /// Enable or disable this profile.
    pub enable: Option<bool>,
    /// The URL or text encoded into the QR code.
    pub url: Option<String>,
    /// Target dimensions for the image.
    #[serde(default)]
    pub fit: FitConfig,
    /// Quiet-zone margin (in modules).
    pub margin: Option<u32>,
    /// Module shape selection.
    #[serde(default)]
    pub shape: ShapeFlags,
    /// Background color (hex or array).
    pub background: Option<ColorCfg>,
    /// Module color (hex or array).
    pub module: Option<ColorCfg>,
    /// Explicit RGBA background color (if provided separately).
    pub background_rgba: Option<[u8; 4]>,
    /// Explicit RGBA module color (if provided separately).
    pub module_rgba: Option<[u8; 4]>,
}

impl Profile {
    /// Return whether this profile is enabled (`true` by default).
    pub fn is_enabled(&self) -> bool {
        self.enable.unwrap_or(true)
    }

    /// Resolve the background [`Color`] using precedence:
    /// 1. `background` hex/array field  
    /// 2. `background_rgba` fallback
    pub fn background_color(&self) -> Option<Color> {
        if let Some(c) = &self.background {
            Some(c.to_color())
        } else {
            self.background_rgba.map(Color::from)
        }
    }

    /// Resolve the module (foreground) [`Color`] using precedence:
    /// 1. `module` hex/array field  
    /// 2. `module_rgba` fallback
    pub fn module_color(&self) -> Option<Color> {
        if let Some(c) = &self.module {
            Some(c.to_color())
        } else {
            self.module_rgba.map(Color::from)
        }
    }
}

/// Top-level configuration block `[preprocessor.qr]` in `book.toml`.
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "kebab-case")]
pub struct QrConfig {
    pub enable: Option<bool>,
    pub url: Option<String>,
    pub qr_path: Option<String>,

    #[serde(default)]
    pub fit: FitConfig,
    pub margin: Option<u32>,
    #[serde(default)]
    pub shape: ShapeFlags,
    pub background: Option<ColorCfg>,
    pub module: Option<ColorCfg>,
    pub background_rgba: Option<[u8; 4]>,
    pub module_rgba: Option<[u8; 4]>,

    /// Map of named custom profiles (`[preprocessor.qr.custom.*]`).
    #[serde(default)]
    pub custom: BTreeMap<String, Profile>,
}

impl Default for QrConfig {
    /// Create a default configuration with standard mdbook-qr values:
    /// - `enable = true`
    /// - `margin = 2`
    /// - White background / black modules
    /// - Empty `url` and `custom` tables
    fn default() -> Self {
        Self {
            enable: Some(true),
            url: None,
            qr_path: None,
            fit: FitConfig::default(),
            margin: Some(2),
            shape: ShapeFlags::default(),
            background: Some(ColorCfg::Hex("#FFFFFFFF".into())),
            module: Some(ColorCfg::Hex("#000000FF".into())),
            background_rgba: None,
            module_rgba: None,
            custom: BTreeMap::new(),
        }
    }
}

impl QrConfig {
    /// Returns whether the QR preprocessor is enabled (`true` by default).
    pub fn is_enabled(&self) -> bool {
        self.enable.unwrap_or(true)
    }

    /// Construct the base (default) [`Profile`] for this configuration.
    pub fn default_profile(&self) -> Profile {
        Profile {
            marker: Some("{{QR_CODE}}".to_string()),
            qr_path: self.qr_path.clone(),
            enable: self.enable,
            url: self.url.clone(),
            fit: self.fit.clone(),
            margin: self.margin,
            shape: self.shape.clone(),
            background: self.background.clone(),
            module: self.module.clone(),
            background_rgba: self.background_rgba,
            module_rgba: self.module_rgba,
        }
    }

    /// Combine a `base` profile with a `child`, inheriting any missing fields.
    ///
    /// Marker and `qr_path` are **not inherited**.
    fn inherit(base: &Profile, child: &Profile) -> Profile {
        Profile {
            marker: child.marker.clone(),
            qr_path: child.qr_path.clone(),
            enable: child.enable.or(base.enable),
            url: child.url.clone().or_else(|| base.url.clone()),
            fit: FitConfig {
                width: child.fit.width.or(base.fit.width),
                height: child.fit.height.or(base.fit.height),
            },
            margin: child.margin.or(base.margin),
            shape: if child.shape.any_set() {
                child.shape.clone()
            } else {
                base.shape.clone()
            },
            background: child.background.clone().or_else(|| base.background.clone()),
            module: child.module.clone().or_else(|| base.module.clone()),
            background_rgba: child.background_rgba.or(base.background_rgba),
            module_rgba: child.module_rgba.or(base.module_rgba),
        }
    }

    /// Log a warning (once per build) for invalid custom profiles
    /// that do not define a `marker` field.
    pub fn warn_invalid_customs(&self) {
        for (name, p) in &self.custom {
            if p.marker.is_none() {
                warn!("custom '{name}' has no `marker`; skipping.");
            }
        }
    }

    /// Return all **valid** profiles, including the default one,
    /// merging inherited values but skipping warnings.
    pub fn profiles(&self) -> Vec<Profile> {
        let base = self.default_profile();
        let mut out = Vec::with_capacity(1 + self.custom.len());
        out.push(base.clone());
        for (_name, p) in &self.custom {
            if p.marker.is_some() {
                out.push(Self::inherit(&base, p));
            }
        }
        out
    }

    /// Detect and return the first duplicate marker among the provided profiles.
    ///
    /// Returns `Some(marker)` if a duplicate exists, otherwise `None`.
    pub fn duplicate_marker_from<'a>(
        profiles: impl IntoIterator<Item = &'a Profile>,
    ) -> Option<String> {
        let mut seen: HashSet<String> = HashSet::new();
        for p in profiles {
            if let Some(m) = p.marker.as_ref() {
                if !seen.insert(m.clone()) {
                    return Some(m.clone());
                }
            }
        }
        None
    }
}
