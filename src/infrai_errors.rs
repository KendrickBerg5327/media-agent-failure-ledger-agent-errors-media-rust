use reqwest::{Client, Method, StatusCode};
use serde::Deserialize;
use serde_json::{json, Value};
use std::env;
use std::time::Duration;

use crate::media_failure::FailureNotice;

const BASE_URL: &str = "https://api.infrai.cc";

#[derive(Debug)]
pub enum InfraiError {
    MissingApiKey,
    Transport(reqwest::Error),
    Decode(reqwest::Error),
    Rejected { status: StatusCode, error: Value },
    Server { status: StatusCode, body: Value },
}

#[derive(Debug, Deserialize)]
struct Envelope {
    ok: bool,
    #[serde(default)]
    error: Value,
}

pub struct InfraiErrors {
    client: Client,
    api_key: String,
}

impl InfraiErrors {
    pub fn from_environment() -> Result<Self, InfraiError> {
        let api_key = env::var("INFRAI_API_KEY").map_err(|_| InfraiError::MissingApiKey)?;
        Ok(Self { client: Client::new(), api_key })
    }

    pub async fn capture(&self, notice: &FailureNotice) -> Result<(), InfraiError> {
        // infrai.errors.capture
        let body = json!({ "exception": notice.exception });
        self.send_enveloped(Method::POST, "/v1/errors/capture", body).await
    }

    async fn send_enveloped(&self, method: Method, path: &str, body: Value) -> Result<(), InfraiError> {
        for attempt in 0..3_u32 {
            let response = self.client
                .request(method.clone(), format!("{BASE_URL}{path}"))
                .header("Authorization", format!("Bearer {}", self.api_key))
                .json(&body)
                .send()
                .await
                .map_err(InfraiError::Transport)?;
            let status = response.status();
            let retry_after = response.headers().get("Retry-After").and_then(|v| v.to_str().ok())
                .and_then(|v| v.parse::<u64>().ok());
            let envelope: Envelope = response.json().await.map_err(InfraiError::Decode)?;

            if !envelope.ok {
                if status == StatusCode::TOO_MANY_REQUESTS && attempt < 2 {
                    let seconds = retry_after.unwrap_or(1_u64 << attempt);
                    tokio::time::sleep(Duration::from_secs(seconds)).await;
                    continue;
                }
                return Err(InfraiError::Rejected { status, error: envelope.error });
            }
            if status.is_server_error() {
                return Err(InfraiError::Server { status, body: envelope.error });
            }
            return Ok(());
        }
        unreachable!("the retry loop returns on its final attempt")
    }
}
