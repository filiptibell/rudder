use anyhow::Result;
use clap::{Parser, Subcommand};
use rudder_http_client::Client;

mod get_ip;
mod start;

#[derive(Debug, Clone, Parser)]
#[command(version, about, long_about = None)]
pub struct Args {
    #[clap(subcommand)]
    pub subcommand: ArgsSubcommand,
}

impl Args {
    pub async fn run(self) -> Result<()> {
        let client = Client::new();
        self.subcommand.run(&client).await
    }
}

#[derive(Debug, Clone, Subcommand)]
pub enum ArgsSubcommand {
    /// Gets the current external IP address for this device
    GetIp(self::get_ip::GetIpCommand),
    /// Starts the DDNS service using the given provider
    Start(self::start::StartCommand),
}

impl ArgsSubcommand {
    pub async fn run(self, client: &Client) -> Result<()> {
        match self {
            Self::GetIp(cmd) => cmd.run(client).await,
            Self::Start(cmd) => cmd.run(client).await,
        }
    }
}
