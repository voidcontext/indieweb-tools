use async_mutex::Mutex;

use super::token_db::TokenDB;
use oauth2::{
    basic::BasicClient, http::HeaderValue, reqwest::async_http_client, AccessToken, RefreshToken,
    TokenResponse,
};
use reqwest::{header::AUTHORIZATION, Client, Request, Response, StatusCode};

struct TokenCredentials {
    access_token: AccessToken,
    refresh_token: RefreshToken,
}

pub struct AuthedClient<DB: TokenDB> {
    oauth_client: BasicClient,
    db: DB,
    provider: String,
    http_client: Client,
    // TODO: do we need this async mutex here? Couldn't we use TokenDB / sled directly?
    tokens: Mutex<TokenCredentials>,
}

impl<DB: TokenDB> AuthedClient<DB> {
    pub fn new(provider: String, oauth_client: BasicClient, db: DB) -> Self {
        let access_token = db
            .get_access_token(&provider)
            .expect("Couldn't load access token");
        let refresh_token = db
            .get_refresh_token(&provider)
            .expect("Couldn't load refresh token");
        Self {
            oauth_client,
            db,
            provider,
            http_client: reqwest::Client::new(),
            tokens: Mutex::new(TokenCredentials {
                access_token,
                refresh_token,
            }),
        }
    }

    pub async fn authed_request(
        &self,
        request: Request,
    ) -> Result<Response, Box<dyn std::error::Error>> {
        let mut request_cloned = request.try_clone().unwrap();
        {
            let tokens = self.tokens.lock().await;
            request_cloned = self.authorize_request(request_cloned, &tokens).await;
        }

        log::debug!("headers: {:?}", request_cloned.headers());
        let response = self.http_client.execute(request_cloned).await?;
        log::debug!("response from execue: {:?}", response);

        if response.status() == StatusCode::UNAUTHORIZED {
            log::debug!("recieved unauthorized response, refresing token");
            let mut tokens = self.tokens.lock().await;
            log::debug!("token credentials lock acquired");
            *tokens = self.exchange_refresh_token(&tokens).await?;

            let request_cloned = request.try_clone().unwrap();
            let request_cloned = self.authorize_request(request_cloned, &tokens).await;
            log::debug!(
                "headers after token refresh: {:?}",
                request_cloned.headers()
            );
            self.http_client
                .execute(request_cloned)
                .await
                .map(|res| {
                    log::debug!("response: {:?}", res);
                    res
                })
                .map_err(|err| Box::new(err) as Box<dyn std::error::Error>)
        } else {
            Ok(response)
        }
    }

    async fn authorize_request(&self, mut request: Request, tokens: &TokenCredentials) -> Request {
        request.headers_mut().append(
            AUTHORIZATION,
            HeaderValue::from_str(&format!("Bearer {}", tokens.access_token.secret())).unwrap(),
        );

        request
    }

    async fn exchange_refresh_token(
        &self,
        tokens: &TokenCredentials,
    ) -> Result<TokenCredentials, Box<dyn std::error::Error>> {
        log::debug!("exchanging refresh_token...");

        let response = self
            .oauth_client
            .exchange_refresh_token(&tokens.refresh_token)
            .request_async(async_http_client)
            .await;

        match response {
            Err(error) => {
                log::error!("token refresh failed: {:?}", error);
                Err(Box::new(error) as Box<dyn std::error::Error>)
            }
            Ok(token_response) => {
                let tokens = TokenCredentials {
                    access_token: token_response.access_token().clone(),
                    refresh_token: token_response
                        .refresh_token()
                        .unwrap_or(&tokens.refresh_token)
                        .clone(),
                };

                log::debug!(
                    "access token refresh successful, new token: {}",
                    tokens.access_token.secret()
                );

                self.db
                    .store(&self.provider, &tokens.access_token, &tokens.refresh_token)
                    .map(|_| tokens)
            }
        }
    }
}

#[cfg(test)]
mod test {
    use crate::auth::token_db::TokenDB;
    use oauth2::{basic::BasicClient, AuthUrl, ClientId, TokenUrl};
    use reqwest::{Method, Request, StatusCode, Url};
    use wiremock::{
        http::HeaderName,
        matchers::{headers, method, path},
        Mock, MockServer, ResponseTemplate,
    };

    use crate::auth::token_db::stubs::StubTokenDB;

    use super::AuthedClient;

    fn basic_client(base_url: &str) -> BasicClient {
        BasicClient::new(
            ClientId::new(String::from("some-client-id")),
            None,
            AuthUrl::new(String::from(format!("{}/oauth/2", base_url))).unwrap(),
            Some(TokenUrl::new(String::from(format!("{}/oauth/token", base_url))).unwrap()),
        )
    }

    fn create_authed_client(base_url: &str) -> AuthedClient<StubTokenDB> {
        AuthedClient::new(
            String::from("acme-inc"),
            basic_client(base_url),
            StubTokenDB::new(),
        )
    }

    #[tokio::test]
    async fn test_token_is_not_refreshed_if_response_is_not_401() {
        let mock_server = MockServer::start().await;

        let authed_client = create_authed_client(&mock_server.uri());

        Mock::given(method("GET"))
            .and(path("/restricted"))
            .respond_with(ResponseTemplate::new(200))
            .mount(&mock_server)
            .await;

        let request = Request::new(
            Method::GET,
            Url::parse(format!("{}/restricted", mock_server.uri()).as_str()).unwrap(),
        );

        let result = authed_client.authed_request(request).await;
        // There is a response
        assert!(result.is_ok(), "{:?}", result);

        // Response is expected
        assert_eq!(result.unwrap().status(), StatusCode::OK);

        let requests = mock_server
            .received_requests()
            .await
            .expect("Requests expected");

        // There was exactly 1 requests
        assert_eq!(requests.len(), 1);

        // The first request was to GET the /restricted url
        assert_eq!(requests[0].url.path(), "/restricted");
        assert_eq!(requests[0].method, wiremock::http::Method::Get);
    }

    #[tokio::test]
    async fn test_token_is_refreshed_if_response_is_401() {
        let mock_server = MockServer::start().await;

        let authed_client = create_authed_client(&mock_server.uri());

        Mock::given(method("GET"))
            .and(path("/restricted"))
            .and(headers(
                "Authorization",
                vec!["Bearer initial-access-token"],
            ))
            .respond_with(ResponseTemplate::new(401))
            .mount(&mock_server)
            .await;

        Mock::given(method("GET"))
            .and(path("/restricted"))
            .and(headers("Authorization", vec!["Bearer new-access-token"]))
            .respond_with(ResponseTemplate::new(200))
            .mount(&mock_server)
            .await;

        Mock::given(method("POST"))
            .and(path("/oauth/token"))
            .respond_with(
                ResponseTemplate::new(200).set_body_raw(
                    r#"
            {
                "token_type":"bearer",
                "expires_in":7200,
                "access_token":"new-access-token",
                "scope":"some-scope",
                "refresh_token":"new-refresh-token"
            }
            "#
                    .as_bytes()
                    .to_owned(),
                    "application/json",
                ),
            )
            .mount(&mock_server)
            .await;

        let request = Request::new(
            Method::GET,
            Url::parse(format!("{}/restricted", mock_server.uri()).as_str()).unwrap(),
        );

        let result = authed_client.authed_request(request).await;
        // There is a response
        assert!(result.is_ok(), "{:?}", result);

        // Response is expected
        assert_eq!(result.unwrap().status(), StatusCode::OK);

        let requests = mock_server
            .received_requests()
            .await
            .expect("Requests expected");

        // There were exactly 3 requests
        assert_eq!(requests.len(), 3);

        // The first request was to GET the /restricted url
        assert_eq!(requests[0].url.path(), "/restricted");
        assert_eq!(requests[0].method, wiremock::http::Method::Get);
        assert_eq!(
            requests[0]
                .headers
                .get(&HeaderName::from("Authorization"))
                .map(|vs| vs.iter().map(|v| v.to_string()).collect::<Vec<_>>()),
            Some(vec!["Bearer initial-access-token".to_string()])
        );

        // The second request was POST-ed to the oauth endpoint
        assert_eq!(requests[1].url.path(), "/oauth/token");
        assert_eq!(requests[1].method, wiremock::http::Method::Post);

        // The request to the auth endpoint had the right credentials
        let form: Vec<(String, String)> = url::form_urlencoded::parse(&requests[1].body)
            .into_owned()
            .collect();
        assert!(form
            .iter()
            .find(|(k, v)| k == "refresh_token" && v == "initial-refresh-token")
            .is_some());
        assert!(form
            .iter()
            .find(|(k, v)| k == "client_id"
                && v == &authed_client.oauth_client.client_id().to_string())
            .is_some());

        // The thirs request was to GET the /restricted url
        assert_eq!(requests[2].url.path(), "/restricted");
        assert_eq!(requests[2].method, wiremock::http::Method::Get);
        assert_eq!(
            requests[2]
                .headers
                .get(&HeaderName::from("Authorization"))
                .map(|vs| vs.iter().map(|v| v.to_string()).collect::<Vec<_>>()),
            Some(vec!["Bearer new-access-token".to_string()])
        );

        // The tokens are updated in the db
        assert_eq!(
            "new-access-token",
            authed_client
                .db
                .get_access_token("acme-inc")
                .unwrap()
                .secret(),
        );

        assert_eq!(
            "new-refresh-token",
            authed_client
                .db
                .get_refresh_token("acme-inc")
                .unwrap()
                .secret(),
        );
    }
}