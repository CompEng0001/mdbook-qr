#![doc = include_str!("../README.md")]

pub mod config;
mod html;
mod image;
mod preprocessor;
mod url;
mod util;

pub use preprocessor::{run_preprocessor_once, QrPreprocessor};
