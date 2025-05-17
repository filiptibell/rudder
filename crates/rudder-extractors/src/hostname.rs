use std::{
    borrow::Borrow,
    collections::HashMap,
    fmt::{self, Display},
    ops::Deref,
    str::FromStr,
};

use axum::{
    extract::{FromRequestParts, Query},
    http::{HeaderName, StatusCode, request::Parts},
};

use idna::uts46::{AsciiDenyList, DnsLength, Hyphens, Uts46};

const HOSTNAME_QUERY_PARAMS: [&str; 7] = [
    "hostname",
    "Hostname",
    "targethostname",
    "target-hostname",
    "target_hostname",
    "targetHostname",
    "TargetHostname",
];

const HOSTNAME_HEADERS: [HeaderName; 3] = [
    HeaderName::from_static("x-hostname"),
    HeaderName::from_static("x-targethostname"),
    HeaderName::from_static("x-target-hostname"),
];

/**
    A hostname extracted from one of the following, in priority order:

    1. A query parameter named one of: `hostname`, `target-hostname`
    2. A header named one of: `x-hostname`, `x-target-hostname`

    Query parameters and headers also support additional casing variants,
    more specifically `PascalCase`, `kebab-case`, and `snake_case`.

    # Hostname Validity

    All `Hostname` instances are valid according to the IDNA uts46
    specification, and can only be constructed from host names
    that are valid to use in A / AAAA records for dynamic DNS.
*/
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Hostname {
    inner: String,
}

impl Deref for Hostname {
    type Target = str;
    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl AsRef<str> for Hostname {
    fn as_ref(&self) -> &str {
        &self.inner
    }
}

impl Borrow<str> for Hostname {
    fn borrow(&self) -> &str {
        &self.inner
    }
}

impl Display for Hostname {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.inner.fmt(f)
    }
}

impl FromStr for Hostname {
    type Err = idna::Errors;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let s = trim_http_prefix(s);
        Uts46::new()
            .to_ascii(
                s.as_bytes(),
                AsciiDenyList::URL,
                Hyphens::Allow,
                DnsLength::VerifyAllowRootDot,
            )
            .map(|d| Hostname {
                inner: d.into_owned(),
            })
    }
}

impl<S> FromRequestParts<S> for Hostname
where
    S: Send + Sync,
{
    type Rejection = (StatusCode, String);

    async fn from_request_parts(parts: &mut Parts, _state: &S) -> Result<Self, Self::Rejection> {
        // 1. First, check all the possible query parameters, in order
        if let Ok(Query(params)) = Query::<HashMap<String, String>>::try_from_uri(&parts.uri) {
            for param in HOSTNAME_QUERY_PARAMS {
                if let Some(value) = params.get(param) {
                    return parse_hostname("query parameter", param, value);
                }
            }
        }

        // 2. Second, check all the possible headers, in order
        for header in HOSTNAME_HEADERS {
            if let Some(value) = parts.headers.get(&header).cloned() {
                let value = value.to_str().map_err(|e| {
                    (
                        StatusCode::BAD_REQUEST,
                        format!("invalid UTF8 in header '{header}': {e}"),
                    )
                })?;
                return parse_hostname("header", header, value);
            }
        }

        // 3. No query parameters or headers were found - definitive user error
        Err((
            StatusCode::BAD_REQUEST,
            String::from("no hostname found in query parameters or headers"),
        ))
    }
}

fn parse_hostname(
    kind: &'static str,
    name: impl Display,
    value: &str,
) -> Result<Hostname, (StatusCode, String)> {
    let value = trim_http_prefix(value);

    if value.is_empty() {
        return Err((
            StatusCode::BAD_REQUEST,
            format!("invalid hostname provided in {kind} '{name}': hostname is empty"),
        ));
    }

    Hostname::from_str(value).map_err(|e| {
        (
            StatusCode::BAD_REQUEST,
            format!("invalid hostname in {kind} '{name}': {e}"),
        )
    })
}

fn trim_http_prefix(value: &str) -> &str {
    let mut current = value.trim();
    loop {
        if let Some(after) = current.strip_prefix("http://") {
            current = after.trim_start();
        } else if let Some(after) = current.strip_prefix("https://") {
            current = after.trim_start();
        } else {
            break;
        }
    }
    current
}
