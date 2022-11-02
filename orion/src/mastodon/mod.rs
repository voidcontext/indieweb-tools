use std::rc::Rc;

use crate::{social::Network, syndicated_post::SyndicatedPost, target::Target};
use async_trait::async_trait;
use futures::TryFutureExt;
use iwt_commons::{text, wormhole::WormholeClient};
use oauth2::AccessToken;
use reqwest::Client;
use rss::Item;

pub struct Mastodon<WHClient: WormholeClient> {
    base_uri: String,
    access_token: AccessToken,
    http_client: Client,
    wormhole_client: Rc<WHClient>,
}

impl<WHClient: WormholeClient> Mastodon<WHClient> {
    pub fn new(base_uri: String, access_token: AccessToken, wormhole_client: Rc<WHClient>) -> Self {
        Self {
            base_uri,
            access_token,
            http_client: Client::new(),
            wormhole_client,
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
impl<WHClient: WormholeClient> Target for Mastodon<WHClient> {
    async fn publish<'a>(
        &self,
        post: &Item,
    ) -> Result<SyndicatedPost, Box<dyn std::error::Error + 'a>> {
        log::debug!("processing post: {:?}", post);

        let permashort_citation = self
            .wormhole_client
            .put_uri(post.link.as_ref().unwrap())
            .await?;

        let status = text::shorten_with_permashort_citation(
            post.description().unwrap(),
            500,
            &permashort_citation,
        );

        self.http_client
            // TODO: make mastodon instance configurable
            .post(format!("{}/api/v1/statuses", self.base_uri))
            .bearer_auth(self.access_token.secret().clone())
            .json(&UpdateStatusRequest { status })
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

    fn network(&self) -> Network {
        Network::Mastodon
    }
}
