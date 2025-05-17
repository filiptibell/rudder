use axum::{Router, http::StatusCode, response::Result, routing::any};

use rudder_extractors::{Hostname, IpVariant};

use crate::auth::EmailAndToken;

pub fn router() -> Router {
    Router::new().fallback(any(root))
}

pub async fn root(
    _auth: EmailAndToken,
    name: Hostname,
    ip: IpVariant,
) -> Result<String, (StatusCode, String)> {
    let ip = match ip {
        IpVariant::Ip(ip) | IpVariant::Auto(ip) => ip,
        IpVariant::Fetch => {
            return Err((
                StatusCode::BAD_REQUEST,
                String::from("ip 'fetch' option can not be used in a Cloudflare Worker"),
            ));
        }
    };

    // TODO: Implement cloudflare client and verify token + send request to update DNS

    Ok(format!(
        "Parsed dynamic DNS request successfully!\
		\n- Hostname: {name}\
		\n- IP: {ip}",
    ))
}
