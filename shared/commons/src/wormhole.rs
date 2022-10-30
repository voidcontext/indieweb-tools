use async_trait::async_trait;
use reqwest::{Client, StatusCode};

struct WormholeClientError {}

impl From<reqwest::Error> for WormholeClientError {
    fn from(_: reqwest::Error) -> Self {
        todo!()
    }
}

#[async_trait(?Send)]
trait WormholeClient {
    async fn put_uri(&self, uri: &str) -> Result<String, WormholeClientError>;
}

pub struct ReqwestWormholeClient<'a> {
    base_uri: &'a str,
    client: Client,
}

impl<'a> ReqwestWormholeClient<'a> {
    fn new(base_uri: &'a str) -> Self {
        Self {
            base_uri,
            client: Client::new(),
        }
    }
}

#[async_trait(?Send)]
impl<'a> WormholeClient for ReqwestWormholeClient<'a> {
    async fn put_uri(&self, uri: &str) -> Result<String, WormholeClientError> {
        let response = self
            .client
            .put(format!("{}/u/{}", self.base_uri, urlencoding::encode(uri)))
            .send()
            .await?;

        if response.status() == StatusCode::OK {
            Ok(response.text().await?)
        } else {
            Err(WormholeClientError {})
        }
    }
}
