use std::{net::IpAddr, time::Duration};

use anyhow::{Context, Result, bail};
use clap::Parser;
use igd_next::{SearchOptions, aio::tokio::search_gateway};
use tokio::time::{MissedTickBehavior, interval};

use rudder_extractors::Hostname;
use rudder_http_client::{
    Client,
    models::cloudflare::{CloudflareDnsRecord, CloudflareDnsRecordKind},
};

/// Starts the DDNS service using the Cloudflare provider
#[derive(Debug, Clone, Parser)]
pub struct CloudflareCommand {
    /// The API token (not key) to use for Cloudflare API authentication
    #[clap(long, env = "CLOUDFLARE_API_TOKEN")]
    pub token: String,
    /// The hostname to use for the DDNS service
    #[clap(long, env = "CLOUDFLARE_HOSTNAME")]
    pub hostname: Hostname,
}

impl CloudflareCommand {
    pub async fn run(self, client: &Client) -> Result<()> {
        tracing::info!(
            "Starting up Cloudflare DDNS service for hostname '{}'",
            self.hostname
        );

        // 1. Make sure we got a valid API token to use
        let cf = client.cloudflare(self.token)?;
        cf.verify_token()
            .await
            .context("failed to verify given api token")?;
        tracing::info!("Verified API token successfully");

        // 2. Extract the single zone that the API token should be assigned to
        let mut zones = cf
            .list_zones()
            .await
            .context("failed to list zones for given api token")?;
        if zones.is_empty() {
            bail!("given api token is not assigned to any zones");
        } else if zones.len() > 1 {
            bail!("given api token is assigned to multiple zones");
        }
        let zone = zones.pop().unwrap();
        tracing::info!(
            id = %zone.id,
            name = %zone.name,
            "Found assigned zone successfully",
        );

        // 3. Set up an interval for checking IP address regularly
        let mut ticker = interval(Duration::from_secs_f64(15.0));
        ticker.set_missed_tick_behavior(MissedTickBehavior::Delay);

        let mut last_ip = None::<IpAddr>;
        loop {
            ticker.tick().await;

            // 4. Find the current gateway / router through uPnP, then external IP address
            let gateway = search_gateway(SearchOptions::default())
                .await
                .context("failed to find gateway / router through uPnP")?;
            let ip = gateway
                .get_external_ip()
                .await
                .context("failed to get external ip through gateway")?;

            // 5. Update the DNS record if the IP has changed
            if last_ip.is_none_or(|last| ip != last) {
                last_ip.replace(ip);

                tracing::info!(ip = %ip, "Updating DNS records with current IP");

                let desired_kind = CloudflareDnsRecordKind::from(ip);
                let desired_name = self.hostname.to_string();

                // 5a. Look for existing DNS record, to see if we should update instead of creating new
                let existing_records = cf
                    .list_dns_records(&zone.id)
                    .await
                    .context("failed to fetch current dns records")?;
                let existing_record = existing_records
                    .into_iter()
                    .find(|record| record.name == desired_name && record.kind == desired_kind);

                // 5b. Update or create the record
                if let Some(existing) = existing_record {
                    if existing.content == ip.to_string() {
                        tracing::info!("No DNS record changes necessary");
                        continue;
                    }

                    tracing::info!(
                        kind = ?desired_kind,
                        name = %desired_name,
                        content = %ip,
                        "Updating existing DNS record"
                    );

                    let mut record = existing.clone();
                    record.content = ip.to_string();

                    cf.update_dns_record(&zone.id, &existing.id, record)
                        .await
                        .context("failed to update dns record")?;

                    tracing::info!("Updated existing DNS record successfully");
                } else {
                    tracing::info!(
                        kind = ?desired_kind,
                        name = %desired_name,
                        content = %ip,
                        "Creating new DNS record"
                    );

                    let record = CloudflareDnsRecord {
                        kind: desired_kind,
                        name: desired_name,
                        content: ip.to_string(),
                        ..Default::default()
                    };

                    cf.create_dns_record(&zone.id, record)
                        .await
                        .context("failed to create dns record")?;

                    tracing::info!("Created new DNS record successfully");
                }
            }
        }
    }
}
