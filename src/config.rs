use serde::{Deserialize, Serialize};
use fast_qr::convert::{Shape, Color};
use std::collections::{BTreeMap, HashSet};
use log::warn;


#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum FailureMode {
    Continue,
    Bail,
}

impl Default for FailureMode {
    fn default() -> Self { FailureMode::Continue }
}

/// Flexible color input accepted in TOML: hex string or RGB/RGBA arrays.
///
/// Examples:
/// - `"#000"` or `"#000000"`
/// - `"#000000FF"`
/// - `[0, 0, 0]`
/// - `[0, 0, 0, 255]`
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(untagged)]
pub enum ColorCfg {
    Hex(String),
    Rgba([u8; 4]),
    Rgb([u8; 3]),
}

impl ColorCfg {
    #[inline]
    pub fn to_color(&self) -> Color {
        match self {
            ColorCfg::Hex(s)   => Color::from(s.as_str()),
            ColorCfg::Rgba(a4) => Color::from(*a4),
            ColorCfg::Rgb(a3)  => Color::from(*a3),
        }
    }
}

/// Optional fit for the injected <img> (px).
#[derive(Clone, Debug, Deserialize, Serialize, Default)]
pub struct FitConfig {
    pub width: Option<u32>,
    pub height: Option<u32>,
}

/// Boolean flags for QR module shape (first-true precedence).
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
    pub fn to_shape(&self) -> Shape {
        if self.circle { Shape::Circle }
        else if self.rounded_square { Shape::RoundedSquare }
        else if self.vertical { Shape::Vertical }
        else if self.horizontal { Shape::Horizontal }
        else if self.diamond { Shape::Diamond }
        else { Shape::Square }
    }

    fn any_set(&self) -> bool {
        self.square || self.circle || self.rounded_square || self.vertical || self.horizontal || self.diamond
    }
}

#[derive(Clone, Debug, Deserialize, Serialize, Default)]
#[serde(rename_all = "kebab-case")]
pub struct Profile {
    /// If None → warn & skip this profile.
    pub marker: Option<String>,
    /// Optional explicit output path for this profile (rel to book src if not absolute).
    pub qr_path: Option<String>,

    pub enable: Option<bool>,
    pub url: Option<String>,
    #[serde(default)]
    pub fit: FitConfig,
    pub margin: Option<u32>,
    #[serde(default)]
    pub shape: ShapeFlags,
    pub background: Option<ColorCfg>,
    pub module: Option<ColorCfg>,
}

impl Profile {
    #[inline]
    pub fn is_enabled(&self) -> bool {
        self.enable.unwrap_or(true)
    }

    /// Resolve the effective background color (from the flexible `background` field).
    #[inline]
    pub fn background_color(&self) -> Option<Color> {
        self.background.as_ref().map(|c| c.to_color())
    }

    /// Resolve the effective module (foreground) color (from the flexible `module` field).
    #[inline]
    pub fn module_color(&self) -> Option<Color> {
        self.module.as_ref().map(|c| c.to_color())
    }
}
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "kebab-case")]
pub struct QrConfig {
    pub enable: Option<bool>,
    pub url: Option<String>,
    pub qr_path: Option<String>,
    #[serde(default)]
    pub on_failure: FailureMode,
    
    #[serde(default)]
    pub include_default: bool,
    
    #[serde(default)]
    pub fit: FitConfig,
    pub margin: Option<u32>,
    #[serde(default)]
    pub shape: ShapeFlags,
    pub background: Option<ColorCfg>,
    pub module: Option<ColorCfg>,


    #[serde(default)]
    pub custom: std::collections::BTreeMap<String, Profile>,
}

impl Default for QrConfig {
    fn default() -> Self {
        Self {
            enable: Some(true),
            url: None,
            qr_path: None,
            on_failure: FailureMode::Continue,
            include_default: true,
            fit: FitConfig::default(),
            margin: Some(2),
            shape: ShapeFlags::default(),
            background: Some(ColorCfg::Hex("#FFFFFFFF".into())),
            module:     Some(ColorCfg::Hex("#000000FF".into())),
            custom: Default::default(),
        }
    }
}

impl QrConfig {
    pub fn is_enabled(&self) -> bool { self.enable.unwrap_or(true) }

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
        }
    }

    /// Inherit missing presentation fields from `base`. Marker & qr_path do NOT inherit.
    pub(crate) fn inherit(base: &Profile, child: &Profile) -> Profile {

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
            shape: if child.shape.any_set() { child.shape.clone() } else { base.shape.clone() },
            background: child.background.clone().or_else(|| base.background.clone()),
            module: child.module.clone().or_else(|| base.module.clone()),
        }
    }

    /// WARN ONCE about invalid customs (marker missing). Does not build profiles.
    pub fn warn_invalid_customs(&self) {
        for (name, p) in &self.custom {
            if p.marker.is_none() {
                warn!("custom '{name}' has no `marker`; skipping.");
            }
        }
    }

    /// Return valid profiles (default + customs that have a marker), **no warnings**.
    pub fn profiles(&self) -> Vec<Profile> {
        let base = self.default_profile();
        let mut out = Vec::with_capacity(1 + self.custom.len());

        if self.include_default {
            out.push(base.clone());
        }
        for (_name, p) in &self.custom {
            if p.marker.is_some() {
                out.push(Self::inherit(&base, p));
            }
        }
        out
    }

    /// Check duplicates among a slice of already-built profiles (valid only).
    pub fn duplicate_marker_from<'a>(profiles: impl IntoIterator<Item = &'a Profile>) -> Option<String> {
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
