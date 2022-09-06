use crate::{syndicated_post::SyndicatedPost, target::Target};
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
            .and_then(|response| async {
                let body = response.text().await;

                log::debug!("response body: {:?}", body);
                todo!()
            })
            .await
            .map_err(|error| Box::new(error) as Box<dyn std::error::Error>)
    }
}
