use std::io::{IsTerminal as _, stderr};

use tracing::level_filters::LevelFilter;
use tracing_subscriber::EnvFilter;

pub fn init_env() {
    dotenvy::dotenv().ok();
}

pub fn init_tracing() {
    let tracing_filter = EnvFilter::builder()
        .with_default_directive(LevelFilter::INFO.into())
        .from_env_lossy();

    tracing_subscriber::fmt()
        .with_env_filter(tracing_filter)
        .with_target(false)
        .with_level(true)
        .with_ansi(stderr().is_terminal())
        .with_writer(stderr)
        .compact()
        .init();
}
