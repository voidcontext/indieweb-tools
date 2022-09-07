use async_trait::async_trait;
use futures::TryFutureExt;
use oauth2::{basic::BasicClient, AuthUrl, ClientId, TokenUrl};
use reqwest::Client;
use rss::Item;

use crate::auth::token_db::TokenDB;
use crate::social::Network;
use crate::syndicated_post::SyndicatedPost;
use crate::{auth::oauth::AuthedClient, target::Target};

pub struct Twitter<DB: TokenDB> {
    authed_client: AuthedClient<DB>,
    http_client: Client,
}

impl<DB: TokenDB> Twitter<DB> {
    pub fn new(client_id: ClientId, db: DB) -> Self {
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
impl<DB: TokenDB> Target for Twitter<DB> {
    async fn publish<'a>(
        &self,
        post: &Item,
    ) -> Result<SyndicatedPost, Box<dyn std::error::Error + 'a>> {
        log::debug!("processing post: {:?}", post);
        let request = self
            .http_client
            .post("https://api.twitter.com/2/tweets")
            .json(&TweetsRequest {
                text: post.description().unwrap().to_owned(),
            });

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
}
