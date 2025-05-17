use std::ops::{Deref, DerefMut};

use axum::{
    extract::FromRequestParts,
    http::{StatusCode, header::AUTHORIZATION, request::Parts},
};

use base64::prelude::*;

/**
    Username + password credentials extracted from a valid `Authorization`
    header that was in the format `Basic base64-username-and-password`

    # Example Usage

    ```rust
    struct MyAuth {
        email: String,
        password: String,
    }

    impl From<(String, String)> for MyAuth {
        fn from((email, password): (String, String)) -> Self {
            Self { email, password }
        }
    }

    async fn handler(auth: BasicAuth<MyAuth>) {
        println!("Email: {}", auth.email);
        println!("Password: {}", auth.password);
    }
    ```
*/
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BasicAuth<T> {
    inner: T,
}

impl<T> BasicAuth<T> {
    pub fn into_inner(self) -> T {
        self.inner
    }
}

impl<T> Deref for BasicAuth<T> {
    type Target = T;
    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl<T> DerefMut for BasicAuth<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.inner
    }
}

impl<S, T> FromRequestParts<S> for BasicAuth<T>
where
    S: Send + Sync,
    T: From<(String, String)>,
{
    type Rejection = (StatusCode, String);

    async fn from_request_parts(parts: &mut Parts, _state: &S) -> Result<Self, Self::Rejection> {
        // 1. Make sure we got an auth header
        let header = parts.headers.get(AUTHORIZATION).ok_or_else(|| {
            (
                StatusCode::UNAUTHORIZED,
                String::from("missing Authorization header"),
            )
        })?;

        // 2. Make sure that we have Basic auth and not something else like Bearer
        let contents = header.as_bytes().strip_prefix(b"Basic ").ok_or_else(|| {
            (
                StatusCode::UNAUTHORIZED,
                String::from(
                    "invalid Authorization header: \
                    must start with 'Basic '",
                ),
            )
        })?;

        // 3. We should now have Basic auth, which is base64-encoded
        let basic_decoded = BASE64_STANDARD.decode(contents).map_err(|e| {
            (
                StatusCode::UNAUTHORIZED,
                format!(
                    "invalid Authorization header: \
                    encountered invalid base64: {e}"
                ),
            )
        })?;
        let basic_str = String::from_utf8(basic_decoded).map_err(|e| {
            (
                StatusCode::UNAUTHORIZED,
                format!(
                    "invalid Authorization header: \
                    invalid UTF-8 after decoding base64: {e}"
                ),
            )
        })?;

        // 4. We decoded successfully, now all that is left is to split username:password
        let (username, password) = basic_str.split_once(':').ok_or_else(|| {
            (
                StatusCode::UNAUTHORIZED,
                String::from(
                    "invalid Authorization header: \
                    missing ':' separator after decoding base64",
                ),
            )
        })?;

        Ok(BasicAuth {
            inner: T::from((username.to_string(), password.to_string())),
        })
    }
}
