use async_mutex::Mutex;

use oauth2::{
    basic::BasicClient, reqwest::async_http_client, AccessToken, RefreshToken, TokenResponse,
};
use reqwest::{header::AUTHORIZATION, Client, Request, Response, StatusCode};

struct AuthedClient {
    oauth_client: BasicClient,
    refresh_token: RefreshToken,
    access_token: Mutex<Option<AccessToken>>,
    http_client: Client,
}

impl AuthedClient {
    fn new(oauth_client: BasicClient, refresh_token: RefreshToken) -> Self {
        Self {
            oauth_client,
            refresh_token,
            http_client: reqwest::Client::new(),
            access_token: Mutex::new(None),
        }
    }

    async fn authed_request(
        &self,
        request: &Request,
    ) -> Result<Response, Box<dyn std::error::Error>> {
        let mut token = self.access_token.lock().await;

        if token.is_none() {
            let new_token = self.exchange_refresh_token().await?;
            *token = Some(new_token);
        }

        let mut request_cloned = request.try_clone().unwrap();
        request_cloned.headers_mut().append(
            AUTHORIZATION,
            format!("Bearer: {}", token.as_ref().unwrap().secret())
                .parse()
                .unwrap(),
        );

        let response = self.http_client.execute(request_cloned).await?;

        if response.status() == StatusCode::UNAUTHORIZED {
            let new_token = self.exchange_refresh_token().await?;
            *token = Some(new_token);

            let mut request_cloned = request.try_clone().unwrap();
            request_cloned.headers_mut().append(
                AUTHORIZATION,
                format!("Bearer: {}", token.as_ref().unwrap().secret())
                    .parse()
                    .unwrap(),
            );
            self.http_client
                .execute(request_cloned)
                .await
                .map_err(|err| Box::new(err) as Box<dyn std::error::Error>)
        } else {
            Ok(response)
        }
    }

    async fn exchange_refresh_token(&self) -> Result<AccessToken, Box<dyn std::error::Error>> {
        println!("refresh token start...");
        let response = self
            .oauth_client
            .exchange_refresh_token(&self.refresh_token)
            .request_async(async_http_client)
            .await?;
        println!("refresh token response received");

        Ok(response.access_token().clone())
    }
}

#[cfg(test)]
mod test {
    use oauth2::{basic::BasicClient, AccessToken, AuthUrl, ClientId, RefreshToken, TokenUrl};
    use reqwest::{Method, Request, StatusCode, Url};
    use wiremock::{
        http::HeaderName,
        matchers::{headers, method, path},
        Mock, MockServer, ResponseTemplate,
    };

    use super::AuthedClient;

    fn basic_client(base_url: &str) -> BasicClient {
        BasicClient::new(
            ClientId::new(String::from("some-client-id")),
            None,
            AuthUrl::new(String::from(format!("{}/oauth/2", base_url))).unwrap(),
            Some(TokenUrl::new(String::from(format!("{}/oauth/token", base_url))).unwrap()),
        )
    }

    fn refresh_token() -> RefreshToken {
        RefreshToken::new(String::from("some-refresh-token"))
    }

    #[tokio::test]
    async fn test_token_is_not_refreshed_if_access_token_available_and_response_is_not_401() {
        let mock_server = MockServer::start().await;

        let authed_client = AuthedClient::new(basic_client(&mock_server.uri()), refresh_token());
        let mut mutex = authed_client.access_token.lock().await;
        *mutex = Some(AccessToken::new(String::from("some-access-token")));
        drop(mutex);

        Mock::given(method("GET"))
            .and(path("/restricted"))
            .respond_with(ResponseTemplate::new(200))
            .mount(&mock_server)
            .await;

        let request = Request::new(
            Method::GET,
            Url::parse(format!("{}/restricted", mock_server.uri()).as_str()).unwrap(),
        );

        let result = authed_client.authed_request(&request).await;
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
    async fn test_initial_access_token_is_requested_if_not_set() {
        let mock_server = MockServer::start().await;

        let authed_client = AuthedClient::new(basic_client(&mock_server.uri()), refresh_token());

        Mock::given(method("POST"))
            .and(path("/oauth/token"))
            .respond_with(
                ResponseTemplate::new(200).set_body_raw(
                    r#"
            {
                "token_type":"bearer",
                "expires_in":7200,
                "access_token":"access-token",
                "scope":"some-scope",
                "refresh_token":"refresh-token"
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

        let result = authed_client.authed_request(&request).await;
        // There is a response
        assert!(result.is_ok(), "{:?}", result);

        let requests = mock_server
            .received_requests()
            .await
            .expect("Requests expected");

        // There was at least 1 requests
        assert!(requests.len() >= 1);

        // The first request was POST-ed to the oauth endpoint
        assert_eq!(requests[0].url.path(), "/oauth/token");
        assert_eq!(requests[0].method, wiremock::http::Method::Post);

        // The request to the auth endpoint had the right credentials
        let form: Vec<(String, String)> = url::form_urlencoded::parse(&requests[0].body)
            .into_owned()
            .collect();
        assert!(form
            .iter()
            .find(|(k, v)| k == "refresh_token" && v == authed_client.refresh_token.secret())
            .is_some());
        assert!(form
            .iter()
            .find(|(k, v)| k == "client_id"
                && v == &authed_client.oauth_client.client_id().to_string())
            .is_some());
    }

    #[tokio::test]
    async fn test_token_is_refreshed_if_response_is_401() {
        let mock_server = MockServer::start().await;

        let authed_client = AuthedClient::new(basic_client(&mock_server.uri()), refresh_token());
        let mut mutex = authed_client.access_token.lock().await;
        *mutex = Some(AccessToken::new(String::from("some-access-token")));
        drop(mutex);

        Mock::given(method("GET"))
            .and(path("/restricted"))
            .and(headers("Authorization", vec!["Bearer: some-access-token"]))
            .respond_with(ResponseTemplate::new(401))
            .mount(&mock_server)
            .await;

        Mock::given(method("GET"))
            .and(path("/restricted"))
            .and(headers("Authorization", vec!["Bearer: new-access-token"]))
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
                "refresh_token":"refresh-token"
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

        let result = authed_client.authed_request(&request).await;
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
            Some(vec!["Bearer: some-access-token".to_string()])
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
            .find(|(k, v)| k == "refresh_token" && v == authed_client.refresh_token.secret())
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
            Some(vec!["Bearer: new-access-token".to_string()])
        );
    }
}
