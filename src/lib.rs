//! Library API for the `mdbook-qr` preprocessor.
//!
//! This crate implements the `mdbook-qr` preprocessor logic, configuration
//! model, and integration entrypoints for mdBook builds.
//!
//! Most users will interact with it indirectly by enabling it in `book.toml`,
//! but the API is documented here for extension and tooling use.

#![cfg_attr(docsrs, feature(doc_cfg))]

/// Configuration types parsed from `[preprocessor.qr]` and its sub-tables.
pub mod config;

mod html;
mod image;
mod preprocessor;
mod url;
mod util;




#[doc(inline)]
pub use html::inject_marker_relative;


#[doc(inline)]
pub use config::{
    ColorCfg, FitConfig, ShapeFlags, Profile, QrConfig,
};


#[doc(inline)]
pub use preprocessor::QrPreprocessor;


#[doc(inline)]
pub use preprocessor::run_preprocessor_once;

