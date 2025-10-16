use clap::{arg, Command};
use std::process;

fn init_logging() {
    if std::env::var_os("RUST_LOG").is_none() {
        unsafe {        
            std::env::set_var("RUST_LOG", "warn,mdbook_qr=debug");
        }
    }
    env_logger::Builder::from_env(env_logger::Env::default())
        .format(|buf, record| {
            use std::io::Write;
            writeln!(
                buf,
                "{} [{}] ({}): {}",
                buf.timestamp_seconds(),
                record.level(),
                record.target(),
                record.args()
            )
        })
        .target(env_logger::Target::Stderr)
        .init();
}

fn main() {
    init_logging();

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
        process::exit(0);
    }

    if let Err(e) = mdbook_qr::run_preprocessor_once() {
        log::error!("preprocessor failed: {e}");
        process::exit(1);
    }
}
