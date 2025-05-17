use anyhow::{Result, anyhow};
use serde::{
    Deserialize, Deserializer,
    de::{DeserializeOwned, Error as SerdeDeError},
};

#[derive(Debug, Clone, Deserialize)]
pub struct CloudflareResponseError {
    code: u32,
    message: String,
    #[serde(default)]
    error_chain: Vec<CloudflareResponseError>,
}

impl CloudflareResponseError {
    fn add_context(self, mut error: anyhow::Error) -> anyhow::Error {
        for chained in self.error_chain.into_iter().rev() {
            error = chained.add_context(error);
        }
        error = error.context(format!("code: {}, message: {}", self.code, self.message));
        error
    }
}

#[derive(Debug, Clone)]
pub enum CloudflareResponse<T> {
    Success {
        result: T,
    },
    Error {
        errors: Vec<CloudflareResponseError>,
    },
}

impl<T> CloudflareResponse<T> {
    pub fn into_result(self) -> Result<T> {
        match self {
            CloudflareResponse::Success { result } => Ok(result),
            CloudflareResponse::Error { errors } => {
                let mut error = anyhow!("cloudflare API error");
                for e in errors {
                    error = e.add_context(error);
                }
                error = error.context("cloudflare API error");
                // FUTURE: Figure out how to pop the first context item off..? the error
                // chain currently both starts and ends with the "cloudflare API error"
                Err(error)
            }
        }
    }
}

impl<'de, T: DeserializeOwned> Deserialize<'de> for CloudflareResponse<T> {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        #[derive(Deserialize)]
        struct CloudflareResponseRaw {
            success: bool,
            result: Option<serde_json::Value>,
            #[serde(default)]
            errors: Vec<CloudflareResponseError>,
        }

        let json = match serde_path_to_error::deserialize::<_, serde_json::Value>(deserializer) {
            Ok(json) => json,
            Err(err) => {
                return Err(SerdeDeError::custom(format!(
                    "response contained invalid json: {err}"
                )));
            }
        };

        match serde_path_to_error::deserialize::<_, CloudflareResponseRaw>(json) {
            Ok(raw) => {
                if raw.success {
                    match raw.result {
                        Some(value) => match serde_path_to_error::deserialize(value) {
                            Ok(result) => Ok(CloudflareResponse::Success { result }),
                            Err(err) => Err(SerdeDeError::custom(format!(
                                "failed to deserialize at '{}': {}",
                                err.path().clone(),
                                err.into_inner(),
                            ))),
                        },
                        None => Err(SerdeDeError::custom(
                            "response was successful, but result is missing",
                        )),
                    }
                } else if raw.errors.is_empty() {
                    Err(SerdeDeError::custom(
                        "response was failure, but errors are missing",
                    ))
                } else {
                    Ok(CloudflareResponse::Error { errors: raw.errors })
                }
            }
            Err(err) => Err(SerdeDeError::custom(err)),
        }
    }
}
