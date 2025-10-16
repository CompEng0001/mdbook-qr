#![doc = include_str!("../README.md")]

pub mod config;
mod preprocessor;
mod image;
mod html;
mod url;
mod util;

pub use preprocessor::{QrPreprocessor, run_preprocessor_once};
