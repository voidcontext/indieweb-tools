use std::fmt::Display;

use async_trait::async_trait;
use reqwest::{Client, StatusCode};

use crate::PermashortCitation;

#[derive(Debug)]
pub struct WormholeClientError {
    pub message: String,
}

impl From<reqwest::Error> for WormholeClientError {
    fn from(e: reqwest::Error) -> Self {
        // TODO: better error handling
        WormholeClientError {
            message: e.to_string(),
        }
    }
}

impl Display for WormholeClientError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!("Wormhole Client Error: {}", self.message))
    }
}

impl std::error::Error for WormholeClientError {}

#[async_trait(?Send)]
pub trait WormholeClient {
    async fn put_uri(&self, uri: &str) -> Result<PermashortCitation, WormholeClientError>;
}

pub struct ReqwestWormholeClient {
    protocol: String,
    domain: String,
    client: Client,
}

impl ReqwestWormholeClient {
    pub fn new(protocol: &str, domain: &str) -> Self {
        Self {
            protocol: protocol.to_owned(),
            domain: domain.to_owned(),
            client: Client::new(),
        }
    }
}

#[async_trait(?Send)]
impl WormholeClient for ReqwestWormholeClient {
    async fn put_uri(&self, uri: &str) -> Result<PermashortCitation, WormholeClientError> {
        let response = self
            .client
            .put(format!(
                "{}://{}/u/{}",
                self.protocol,
                self.domain,
                urlencoding::encode(uri)
            ))
            .send()
            .await?;

        if response.status().is_success() {
            let short = response.text().await?;
            Ok(PermashortCitation::new(
                self.protocol.clone(),
                self.domain.clone(),
                format!("s/{}", short),
            ))
        } else {
            Err(WormholeClientError {
                message: format!("Unexpected status: {}", response.status()),
            })
        }
    }
}
