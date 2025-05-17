use anyhow::Result;
use clap::Parser;

mod command;
mod utils;

use self::command::Args;
use self::utils::{init_env, init_tracing};

#[tokio::main(flavor = "current_thread")]
async fn main() -> Result<()> {
    init_env();
    init_tracing();
    Args::parse().run().await
}
