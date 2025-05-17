use std::{collections::HashMap, fmt::Display, net::IpAddr};

use axum::{
    extract::{FromRequestParts, Query},
    http::{HeaderMap, HeaderName, HeaderValue, StatusCode, request::Parts},
};

const IP_QUERY_PARAMS: [&str; 11] = [
    "ip",
    "Ip",
    "myip",
    "my_ip",
    "myIp",
    "MyIp",
    "targetip",
    "target-ip",
    "target_ip",
    "targetIp",
    "TargetIp",
];

const IP_HEADERS: [HeaderName; 5] = [
    HeaderName::from_static("x-ip"),
    HeaderName::from_static("x-myip"),
    HeaderName::from_static("x-my-ip"),
    HeaderName::from_static("x-targetip"),
    HeaderName::from_static("x-target-ip"),
];

const CF_CONNECTING_IP: HeaderName = HeaderName::from_static("cf-connecting-ip");
const X_REAL_IP: HeaderName = HeaderName::from_static("x-real-ip");
const X_FORWARDED_FOR: HeaderName = HeaderName::from_static("x-forwarded-for");

/**
    An IP variant (either V4 or V6 address, or special value)
    extracted from one of the following, in priority order:

    1. A query parameter named one of: `ip`, `myip`, `targetip`
    2. A header named one of: `x-ip`, `x-myip`, `x-targetip`

    Query parameters and headers also support additional casing variants,
    more specifically `PascalCase`, `kebab-case`, and `snake_case`.

    # Special Values

    ## `auto`

    If a value for IP is present and set to `auto`, the IP address
    will be extracted from one of the following, in priority order:

    1. `CF-Connecting-Ip`
    2. `X-Real-Ip`
    3. `X-Forwarded-For`

    ## `fetch`

    If a value for IP is present and set to `fetch`, the probable
    IP address will automatically be fetched using the free API
    for non-commercial use at `https://ip-api.com`.
*/
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum IpVariant {
    /// An IP address directly provided in a query parameter or header
    Ip(IpAddr),
    /// An IP address extracted from a common header ("auto" option was passed)
    Auto(IpAddr),
    /// Unknown IP address that should be dynamically resolved ("fetch" option was passed)
    Fetch,
}

impl<S> FromRequestParts<S> for IpVariant
where
    S: Send + Sync,
{
    type Rejection = (StatusCode, String);

    async fn from_request_parts(parts: &mut Parts, _state: &S) -> Result<Self, Self::Rejection> {
        // 1. First, check all the possible query parameters, in order
        if let Ok(Query(params)) = Query::<HashMap<String, String>>::try_from_uri(&parts.uri) {
            for param in IP_QUERY_PARAMS {
                if let Some(value) = params.get(param) {
                    return parse_ip_variant(&parts.headers, "query parameter", param, value);
                }
            }
        }

        // 2. Second, check all the possible headers, in order
        for header in IP_HEADERS {
            if let Some(value) = parts.headers.get(&header) {
                return parse_ip_variant(
                    &parts.headers,
                    "header",
                    &header,
                    header_str(&header, value)?,
                );
            }
        }

        // 3. No query parameters or headers were found - definitive user error
        Err((
            StatusCode::BAD_REQUEST,
            String::from("no IP address found in query parameters or headers"),
        ))
    }
}

fn parse_ip_variant(
    headers: &HeaderMap,
    kind: &'static str,
    name: impl Display,
    value: &str,
) -> Result<IpVariant, (StatusCode, String)> {
    let value = value.trim();
    if value.eq_ignore_ascii_case("fetch") {
        Ok(IpVariant::Fetch)
    } else if value.eq_ignore_ascii_case("auto") {
        // 1. CF-Connecting-Ip
        if let Some(value) = headers.get(CF_CONNECTING_IP) {
            let value = header_str(CF_CONNECTING_IP, value)?;
            return parse_ip("header", CF_CONNECTING_IP, value).map(IpVariant::Auto);
        }

        // 2. X-Real-Ip
        if let Some(value) = headers.get(X_REAL_IP) {
            let value = header_str(X_REAL_IP, value)?;
            return parse_ip("header", X_REAL_IP, value).map(IpVariant::Auto);
        }

        // 3. X-Forwarded-For
        if let Some(value) = headers.get(X_FORWARDED_FOR) {
            let value = header_str(X_FORWARDED_FOR, value)?;
            let first_ip = value.split_once(',').map_or(value, |s| s.0);
            return parse_ip("header", X_FORWARDED_FOR, first_ip).map(IpVariant::Auto);
        }

        Err((
            StatusCode::BAD_REQUEST,
            format!("'auto' specified in {kind} '{name}', but no relevant IP headers were found"),
        ))
    } else {
        parse_ip(kind, name, value).map(IpVariant::Ip)
    }
}

fn parse_ip(
    kind: &'static str,
    name: impl Display,
    value: &str,
) -> Result<IpAddr, (StatusCode, String)> {
    let value = value.trim();

    if value.is_empty() {
        return Err((
            StatusCode::BAD_REQUEST,
            format!("empty IP address in {kind} '{name}'"),
        ));
    }

    value.parse().map_err(|e| {
        (
            StatusCode::BAD_REQUEST,
            format!("invalid IP address in {kind} '{name}': {e}"),
        )
    })
}

fn header_str(
    header_name: impl Display,
    header_value: &HeaderValue,
) -> Result<&str, (StatusCode, String)> {
    header_value.to_str().map_err(|e| {
        (
            StatusCode::BAD_REQUEST,
            format!("invalid UTF8 in header '{header_name}': {e}"),
        )
    })
}
