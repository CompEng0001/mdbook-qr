use clap::{Command, arg};
use std::process;

fn main() {
    let cli = Command::new("mdbook-qr")
        .about("An mdBook preprocessor that injects QR codes into pages")
        .version(env!("CARGO_PKG_VERSION"))
        .subcommand(
            Command::new("supports")
                .about("Check if a renderer is supported")
                .arg(arg!(<renderer> "Renderer name")),
        );

    let matches = cli.get_matches();

    if let Some(("supports", _)) = matches.subcommand() {
        // Supports all renderers
        process::exit(0);
    }

    if let Err(e) = mdbook_qr::run_preprocessor_once() {
        log::error!("preprocessor failed: {e}");
        process::exit(1);
    }
}
