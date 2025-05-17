use axum::{
    extract::FromRequestParts,
    http::{StatusCode, request::Parts},
};

use rudder_extractors::BasicAuth;

#[derive(Debug, Clone)]
pub struct EmailAndToken {
    pub email: String,
    pub token: String,
}

impl From<(String, String)> for EmailAndToken {
    fn from((email, token): (String, String)) -> Self {
        Self { email, token }
    }
}

impl<S> FromRequestParts<S> for EmailAndToken
where
    S: Send + Sync,
{
    type Rejection = (StatusCode, String);

    async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
        BasicAuth::<Self>::from_request_parts(parts, state)
            .await
            .map(BasicAuth::into_inner)
    }
}
