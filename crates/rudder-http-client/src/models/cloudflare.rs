use std::net::IpAddr;

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum CloudflareUserTokenStatus {
    Active,
    Disabled,
    Expired,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CloudflareUserToken {
    pub status: CloudflareUserTokenStatus,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CloudflareZone {
    pub id: String,
    pub name: String,
    pub account: CloudflareZoneAccount,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CloudflareZoneAccount {
    pub id: String,
    pub name: String,
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "UPPERCASE")]
pub enum CloudflareDnsRecordKind {
    #[default]
    A,
    AAAA,
    CAA,
    CERT,
    CNAME,
    DNSKEY,
    DS,
    HTTPS,
    LOC,
    MX,
    NAPTR,
    NS,
    OPENPGPKEY,
    PTR,
    SMIMEA,
    SRV,
    SSHFP,
    SVCB,
    TLSA,
    TXT,
    URI,
}

impl From<IpAddr> for CloudflareDnsRecordKind {
    fn from(ip: IpAddr) -> Self {
        match ip {
            IpAddr::V4(_) => CloudflareDnsRecordKind::A,
            IpAddr::V6(_) => CloudflareDnsRecordKind::AAAA,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CloudflareDnsRecord {
    #[serde(default, skip_serializing)]
    pub id: String,
    #[serde(default, rename = "type")]
    pub kind: CloudflareDnsRecordKind,
    #[serde(default)]
    pub name: String,
    #[serde(default)]
    pub content: String,
    #[serde(default)]
    pub comment: Option<String>,
    #[serde(default)]
    pub proxied: bool,
    #[serde(default = "default_ttl")]
    pub ttl: u32,
}

impl Default for CloudflareDnsRecord {
    fn default() -> Self {
        Self {
            id: String::new(),
            kind: CloudflareDnsRecordKind::default(),
            name: String::new(),
            content: String::new(),
            comment: None,
            proxied: false,
            ttl: default_ttl(),
        }
    }
}

fn default_ttl() -> u32 {
    3600
}
