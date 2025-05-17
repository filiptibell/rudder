use std::{
    net::{IpAddr, SocketAddr},
    time::Duration,
};

use anyhow::{Context, Result};
use clap::Parser;
use igd_next::{SearchOptions, aio::tokio::search_gateway};
use tokio::time::{MissedTickBehavior, interval};

use rudder_http_client::Client;

/// Gets the current external IP address for this device
#[derive(Debug, Clone, Parser)]
pub struct GetIpCommand {
    /// Whether to watch for IP address changes or not
    #[clap(short, long, default_value_t = false)]
    pub watch: bool,
    /// How often to check for IP address changes (in seconds)
    #[clap(short, long, default_value_t = 15.0)]
    pub interval: f64,
    /// How long before timeout for getting the IP occurs (in seconds)
    #[clap(short, long, default_value_t = 10.0)]
    pub timeout: f64,
}

impl GetIpCommand {
    pub async fn run(self, _client: &Client) -> Result<()> {
        let interval_dur = Duration::from_secs_f64(self.interval);
        let timeout_dur = Duration::from_secs_f64(self.timeout);

        // 1. Set up an interval for checking IP address regularly,
        //    if watch mode is not enabled this will fire once
        //    instantly and we only go through one iteration
        let mut ticker = interval(interval_dur);
        ticker.set_missed_tick_behavior(MissedTickBehavior::Delay);

        let mut last_gate = None::<SocketAddr>;
        let mut last_ip = None::<IpAddr>;
        loop {
            ticker.tick().await;

            // 2a. Find the current gateway / router through uPnP
            let options = SearchOptions {
                timeout: Some(timeout_dur),
                ..Default::default()
            };
            let gateway = search_gateway(options)
                .await
                .context("failed to find gateway / router through uPnP")?;
            let gate = gateway.addr;

            // 2b. Emit a message if it was found or changed
            if last_gate.is_none_or(|last| gate != last) {
                if last_gate.is_some() {
                    println!("Changed gateway / router: {gate}");
                } else {
                    println!("Found gateway / router: {gate}");
                }
            }

            // 3a. Find the current external IP address through the gateway
            let ip = gateway
                .get_external_ip()
                .await
                .context("failed to get external ip through gateway")?;

            // 3b. Emit a message if it was found or changed
            if last_ip.is_none_or(|last| ip != last) {
                if last_ip.is_some() {
                    println!("Changed external IP: {ip}");
                } else {
                    println!("Found external IP: {ip}");
                }
            }

            // 4. Store the last known gateway address and external IP
            last_gate.replace(gateway.addr);
            last_ip.replace(ip);

            // 5. Keep watching for changes if requested, otherwise exit
            if !self.watch {
                break;
            }
        }

        Ok(())
    }
}
