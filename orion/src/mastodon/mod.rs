use crate::target::Target;
use async_trait::async_trait;
use futures::{StreamExt, TryFutureExt};
use oauth2::AccessToken;
use reqwest::Client;
use rss::Item;

pub struct Mastodon {
    access_token: AccessToken,
    http_client: Client,
}

impl Mastodon {
    pub fn new(access_token: AccessToken) -> Self {
        Self {
            access_token,
            http_client: Client::new(),
        }
    }
}

#[derive(serde::Serialize)]
struct UpdateStatusRequest {
    status: String,
}

#[async_trait(?Send)]
impl Target for Mastodon {
    async fn publish<'a>(&self, post: &Item) -> Result<(), Box<dyn std::error::Error + 'a>> {
        log::debug!("processing post: {:?}", post);
        self.http_client
            // TODO: make mastodon instance configurable
            .post("https://mastodon.social/api/v1/statuses")
            .bearer_auth(self.access_token.secret().clone())
            .json(&UpdateStatusRequest {
                status: post.description().unwrap().to_owned(),
            })
            .send()
            .and_then(|response| async {
                let body = response.text().await;

                log::debug!("response body: {:?}", body);
                Ok(())
            })
            .await
            .map_err(|error| Box::new(error) as Box<dyn std::error::Error>)
    }
}
