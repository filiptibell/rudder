use std::sync::Arc;

use anyhow::{Context, Result, bail};
use reqwest::header::AUTHORIZATION;

use crate::{
    models::cloudflare::{
        CloudflareDnsRecord, CloudflareUserToken, CloudflareUserTokenStatus, CloudflareZone,
    },
    private::cloudflare::CloudflareResponse,
};

#[derive(Debug, Clone)]
pub struct CloudflareClient {
    pub(crate) inner: reqwest::Client,
    pub(crate) api_token: Arc<str>,
}

impl CloudflareClient {
    pub async fn verify_token(&self) -> Result<()> {
        let request = self
            .inner
            .get("https://api.cloudflare.com/client/v4/user/tokens/verify")
            .header(AUTHORIZATION, self.api_token.as_ref());
        let response = request
            .send()
            .await
            .context("verifying token for account failed")?;
        let token = response
            .json::<CloudflareResponse<CloudflareUserToken>>()
            .await
            .context("verifying token for account response failure")?
            .into_result()?;
        if !matches!(token.status, CloudflareUserTokenStatus::Active) {
            bail!("token status is {:?}", token.status)
        }
        Ok(())
    }

    pub async fn list_zones(&self) -> Result<Vec<CloudflareZone>> {
        let request = self
            .inner
            .get("https://api.cloudflare.com/client/v4/zones")
            .header(AUTHORIZATION, self.api_token.as_ref());
        let response = request
            .send()
            .await
            .context("listing zones for account failed")?;
        response
            .json::<CloudflareResponse<_>>()
            .await
            .context("listing zones for account response failure")?
            .into_result()
    }

    pub async fn list_dns_records(&self, zone_id: &str) -> Result<Vec<CloudflareDnsRecord>> {
        let request = self
            .inner
            .get(format!(
                "https://api.cloudflare.com/client/v4/zones/{zone_id}/dns_records"
            ))
            .header(AUTHORIZATION, self.api_token.as_ref());
        let response = request
            .send()
            .await
            .context("listing zones for account failed")?;
        response
            .json::<CloudflareResponse<_>>()
            .await
            .context("listing zones for account response failure")?
            .into_result()
    }

    pub async fn create_dns_record(
        &self,
        zone_id: &str,
        record: CloudflareDnsRecord,
    ) -> Result<CloudflareDnsRecord> {
        let request = self
            .inner
            .post(format!(
                "https://api.cloudflare.com/client/v4/zones/{zone_id}/dns_records"
            ))
            .header(AUTHORIZATION, self.api_token.as_ref());
        let response = request
            .json(&record)
            .send()
            .await
            .context("creating dns record failed")?;
        response
            .json::<CloudflareResponse<_>>()
            .await
            .context("creating dns record response failure")?
            .into_result()
    }

    pub async fn update_dns_record(
        &self,
        zone_id: &str,
        record_id: &str,
        record: CloudflareDnsRecord,
    ) -> Result<CloudflareDnsRecord> {
        let request = self
            .inner
            .patch(format!(
                "https://api.cloudflare.com/client/v4/zones/{zone_id}/dns_records/{record_id}"
            ))
            .header(AUTHORIZATION, self.api_token.as_ref());
        let response = request
            .json(&record)
            .send()
            .await
            .context("updating dns record failed")?;
        response
            .json::<CloudflareResponse<_>>()
            .await
            .context("updating dns record response failure")?
            .into_result()
    }
}
