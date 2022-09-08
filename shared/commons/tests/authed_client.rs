use std::rc::Rc;

use iwt_commons::{auth::token_db::TokenDB, social::Network};
use oauth2::{basic::BasicClient, AuthUrl, ClientId, TokenUrl};
use reqwest::{Method, Request, StatusCode, Url};
use wiremock::{
    http::HeaderName,
    matchers::{headers, method, path},
    Mock, MockServer, ResponseTemplate,
};

use iwt_commons_stubs::auth::token_db::stubs::*;

use iwt_commons::auth::oauth::AuthedClient;

fn basic_client(base_url: &str) -> BasicClient {
    BasicClient::new(
        ClientId::new(String::from("some-client-id")),
        None,
        AuthUrl::new(String::from(format!("{}/oauth/2", base_url))).unwrap(),
        Some(TokenUrl::new(String::from(format!("{}/oauth/token", base_url))).unwrap()),
    )
}

fn create_authed_client(base_url: &str) -> (Rc<impl TokenDB>, AuthedClient<impl TokenDB>) {
    let db = StubTokenDB::new();
    let shared_db = Rc::new(db);
    (Rc::clone(&shared_db), AuthedClient::new(Network::Twitter, basic_client(base_url), shared_db))
}

#[tokio::test]
async fn test_token_is_not_refreshed_if_response_is_not_401() {
    let mock_server = MockServer::start().await;

    let (_, authed_client) = create_authed_client(&mock_server.uri());

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

    let (db, authed_client) = create_authed_client(&mock_server.uri());

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
            && v == "some-client-id")
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
        db.get_access_token(&Network::Twitter)
            .unwrap()
            .secret(),
    );

    assert_eq!(
        "new-refresh-token",
        db.get_refresh_token(&Network::Twitter)
            .unwrap()
            .secret(),
    );
}
