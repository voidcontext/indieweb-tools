use std::sync::Arc;

use async_mutex::Mutex;
use async_trait::async_trait;
use oauth2::{basic::BasicClient, AccessToken, AuthUrl, ClientId, TokenUrl};
use reqwest::Client;
use rss::Item;

use crate::target::Target;

pub struct Twitter {
    pub oauth_client: BasicClient,
    pub http_client: Client,
    pub access_token: Arc<Mutex<Option<AccessToken>>>,
}

impl Twitter {
    fn new(client_id: ClientId) -> Self {
        let oauth_client = BasicClient::new(
            client_id,
            None,
            AuthUrl::new("https://twitter.com/i/oauth2/authorize".to_string())
                .expect("Twitter auth url is invalid."),
            Some(
                TokenUrl::new("https://api.twitter.com/2/oauth2/token".to_string())
                    .expect("Twitter token url is invalid"),
            ),
        );

        let http_client = reqwest::Client::new();
        todo!()
    }
}

#[async_trait]
impl Target for Twitter {
    async fn publish<'a>(&self, posts: &[Item]) -> Result<(), Box<dyn std::error::Error + 'a>> {
        todo!()
    }
}
