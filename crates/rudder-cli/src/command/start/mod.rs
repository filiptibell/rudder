use anyhow::Result;
use clap::{Parser, Subcommand};
use rudder_http_client::Client;

mod cloudflare;

/// Starts the DDNS service using the given provider
#[derive(Debug, Clone, Parser)]
pub struct StartCommand {
    #[clap(subcommand)]
    pub subcommand: ArgsSubcommand,
}

impl StartCommand {
    pub async fn run(self, client: &Client) -> Result<()> {
        self.subcommand.run(client).await
    }
}

#[derive(Debug, Clone, Subcommand)]
pub enum ArgsSubcommand {
    /// Starts the DDNS service using the Cloudflare provider
    Cloudflare(self::cloudflare::CloudflareCommand),
}

impl ArgsSubcommand {
    pub async fn run(self, client: &Client) -> Result<()> {
        match self {
            Self::Cloudflare(cmd) => cmd.run(client).await,
        }
    }
}
