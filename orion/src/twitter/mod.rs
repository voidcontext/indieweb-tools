use std::rc::Rc;

use async_trait::async_trait;
use futures::TryFutureExt;
use iwt_commons::text;
use oauth2::{basic::BasicClient, AuthUrl, ClientId, TokenUrl};
use reqwest::Client;
use rss::Item;

use crate::auth::token_db::TokenDB;
use crate::social::Network;
use crate::syndicated_post::SyndicatedPost;
use crate::{auth::oauth::AuthedClient, target::Target};
use iwt_commons::wormhole::WormholeClient;

pub struct Twitter<DB: TokenDB, WHClient: WormholeClient> {
    authed_client: AuthedClient<DB>,
    http_client: Client,
    wormhole_client: Rc<WHClient>,
}

impl<DB: TokenDB, WHClient: WormholeClient> Twitter<DB, WHClient> {
    pub fn new(client_id: ClientId, db: Rc<DB>, wormhole_client: Rc<WHClient>) -> Self {
        Self {
            authed_client: AuthedClient::new(
                Network::Twitter,
                BasicClient::new(
                    client_id,
                    None,
                    AuthUrl::new("https://twitter.com/i/oauth2/authorize".to_string())
                        .expect("Twitter auth url is invalid."),
                    Some(
                        TokenUrl::new("https://api.twitter.com/2/oauth2/token".to_string())
                            .expect("Twitter token url is invalid"),
                    ),
                ),
                db,
            ),
            http_client: Client::new(),
            wormhole_client,
        }
    }
}

#[derive(serde::Serialize)]
struct TweetsRequest {
    text: String,
}

#[derive(serde::Deserialize)]
struct TweetResponse {
    data: TweetResponseData,
}

#[derive(serde::Deserialize)]
struct TweetResponseData {
    id: String,
}

#[async_trait(?Send)]
impl<DB: TokenDB, WHClient: WormholeClient> Target for Twitter<DB, WHClient> {
    async fn publish<'a>(
        &self,
        post: &Item,
    ) -> Result<SyndicatedPost, Box<dyn std::error::Error + 'a>> {
        log::debug!("processing post: {:?}", post);

        let permashort_citation = self
            .wormhole_client
            .put_uri(post.link.as_ref().unwrap())
            .await?;

        let text = text::shorten_with_permashort_citation(
            post.description().unwrap(),
            280,
            &permashort_citation,
        );

        let request = self
            .http_client
            .post("https://api.twitter.com/2/tweets")
            .json(&TweetsRequest { text });

        self.authed_client
            .authed_request(request.build().unwrap())
            .and_then(|response| async {
                let body = response.text().await.expect("Body should be available");

                serde_json::from_str::<TweetResponse>(&body)
                    .map(|response| SyndicatedPost::new(Network::Twitter, &response.data.id, post))
                    .map_err(|err| Box::new(err) as Box<dyn std::error::Error>)
            })
            .await
    }

    fn network(&self) -> Network {
        Network::Twitter
    }
}
