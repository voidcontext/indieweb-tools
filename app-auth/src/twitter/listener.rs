use std::{
    collections::HashMap,
    net::{IpAddr, Ipv4Addr, SocketAddr},
    sync::Arc,
};

use axum::{
    extract::Query,
    response::{Html, IntoResponse},
    routing::get,
    Extension, Router,
};
use serde_derive::Deserialize;
use tokio::sync::mpsc::Sender;

use super::Error;
use crate::Config;

struct State {
    challenge: String,
    oauth_state: String,
    client_id: String,
    shutdown_signal: Sender<()>,
    sled_db_path: Option<String>,
}

pub async fn start(
    config: &Config,
    challenge: &String,
    csrf_state: &String,
    sled_db_path: Option<String>,
) -> Result<(), Error> {
    // Create a channel to be able to shut down the webserver from the
    // Request handler after receiving the auth code
    let (tx, mut rx) = tokio::sync::mpsc::channel::<()>(10);

    // Initialise the shared state
    let state = Arc::new(State {
        challenge: challenge.clone(),
        oauth_state: csrf_state.clone(),
        client_id: config.twitter.client_id.clone(),
        shutdown_signal: tx,
        sled_db_path,
    });

    let sock_addr = SocketAddr::new(IpAddr::V4(Ipv4Addr::LOCALHOST), 6009);
    let app = Router::new()
        .route("/", get(receive_token))
        // shate the state with the request handler
        .layer(Extension(state));

    axum::Server::bind(&sock_addr)
        .serve(app.into_make_service())
        // gracefuly shut down the server when we receive a message on the
        // previously created channel
        .with_graceful_shutdown(async { rx.recv().await.unwrap() })
        .await
        .map_err(|error| match error {
            _ => Error::ListenerError(),
        })
}

#[derive(Deserialize)]
struct TokenResponse {
    token_type: String,
    access_token: String,
    refresh_token: String,
}

async fn receive_token(
    Query(params): Query<HashMap<String, String>>,
    Extension(state): Extension<Arc<State>>,
) -> impl IntoResponse {
    let state_param = params.get("state").expect("state param not found");
    if state_param != &state.oauth_state {
        panic!(
            "Invalid state param,
expected: {}
got     : {}",
            state.oauth_state, state_param
        )
    }

    let auth_code = params.get("code").expect("auth code param not found");
    log::debug!("Got auth code, exchanging for access token");
    log::debug!("auth_code is {}", auth_code);

    let challenge = state.challenge.to_string();
    let params = [
        ("code", auth_code.as_str()),
        ("grant_type", "authorization_code"),
        ("client_id", state.client_id.as_str()),
        ("code_verifier", challenge.as_str()),
        ("redirect_uri", "http://127.0.0.1:6009"),
    ];

    // Exchange the auth code to an access_token and a refresh_token
    let client = reqwest::Client::new();
    let result = client
        .post("https://api.twitter.com/2/oauth2/token")
        .form(&params)
        .send()
        .await
        .expect("Oauth request failed");

    let json = result.text().await.expect("Couldn't get response body");
    log::debug!("json: {}", json);
    let tokens =
        serde_json::from_str::<TokenResponse>(&json).expect("Coulnd't decode json response");

    println!(
        "
token_type: {}
access_token: {}
refresh_token: {}
",
        tokens.token_type, tokens.access_token, tokens.refresh_token
    );

    if let Some(db_path) = state.sled_db_path.clone() {
        // Initialize db to store tokens
        let db = sled::open(db_path).expect("Cannot create / open db");

        db.insert(
            bincode::serialize("auth:twitter:access_token").unwrap(),
            bincode::serialize(&tokens.access_token).unwrap(),
        )
        .unwrap();

        db.insert(
            bincode::serialize("auth:twitter:refresh_token").unwrap(),
            bincode::serialize(&tokens.refresh_token).unwrap(),
        )
        .unwrap();
    }

    // Send the shut down signal
    state.shutdown_signal.send(()).await.unwrap();

    Html("<h1>Hello from twitter-auth</h1><p>Your tokens are displayed on the standard output.</p>")
}
