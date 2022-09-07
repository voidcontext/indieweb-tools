use crate::{social::Network, syndicated_post::SyndicatedPost, target::Target};
use async_trait::async_trait;
use futures::TryFutureExt;
use oauth2::AccessToken;
use reqwest::Client;
use rss::Item;

pub struct Mastodon {
    base_uri: String,
    access_token: AccessToken,
    http_client: Client,
}

impl Mastodon {
    pub fn new(base_uri: String, access_token: AccessToken) -> Self {
        Self {
            base_uri,
            access_token,
            http_client: Client::new(),
        }
    }
}

#[derive(serde::Serialize)]
struct UpdateStatusRequest {
    status: String,
}

#[derive(serde::Deserialize)]
struct MastodonResponse {
    id: String,
}

#[async_trait(?Send)]
impl Target for Mastodon {
    async fn publish<'a>(
        &self,
        post: &Item,
    ) -> Result<SyndicatedPost, Box<dyn std::error::Error + 'a>> {
        log::debug!("processing post: {:?}", post);
        self.http_client
            // TODO: make mastodon instance configurable
            .post(format!("{}/api/v1/statuses", self.base_uri))
            .bearer_auth(self.access_token.secret().clone())
            .json(&UpdateStatusRequest {
                status: post.description().unwrap().to_owned(),
            })
            .send()
            .map_err(|err| Box::new(err) as Box<dyn std::error::Error>)
            .and_then(|response| async {
                let body = response.text().await.expect("Response body expected");

                serde_json::from_str::<MastodonResponse>(&body)
                    .map(|response| SyndicatedPost::new(Network::Mastodon, &response.id, post))
                    .map_err(|err| Box::new(err) as Box<dyn std::error::Error>)
            })
            .await
    }
}
