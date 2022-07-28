use std::{
    collections::HashMap,
    env, fs,
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
use simple_logger::SimpleLogger;
use tokio::sync::mpsc::Sender;

use log::LevelFilter::{Debug, Info};
use rand::{rngs::OsRng, RngCore};

#[derive(Debug, Deserialize)]
struct Config {
    client_id: String,
}

impl Config {
    pub fn from_file(file_name: &str) -> Result<Config, toml::de::Error> {
        let config_str = fs::read_to_string(file_name).unwrap();

        toml::from_str(&config_str)
    }
}

struct State {
    challenge: String,
    oauth_state: String,
    client_id: String,
    shutdown_signal: Sender<()>,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let log_level =
        env::var("TWITTER_AUTH_DEBUG")
            .map_or(Info, |debug| if debug == "1" { Debug } else { Info });
    SimpleLogger::new().with_level(log_level).init().unwrap();

    let config = Config::from_file("config.toml")?;

    let (tx, mut rx) = tokio::sync::mpsc::channel::<()>(10);

    let mut challenge = [0u8; 64];
    let mut state = [0u8; 64];

    OsRng.fill_bytes(&mut challenge);
    OsRng.fill_bytes(&mut state);

    let state = Arc::new(State {
        challenge: base64::encode(challenge),
        oauth_state: base64::encode(state),
        client_id: config.client_id,
        shutdown_signal: tx,
    });

    let oauth_uri = format!(
        concat!(
            "https://twitter.com/i/oauth2/authorize?response_type=code&",
            "client_id={}&redirect_uri={}&scope={}&state={}&code_challenge={}&code_challenge_method=plain",
        ),
        state.client_id,
        "http://127.0.0.1:6009",
        "tweet.read%20tweet.write%20users.read%20offline.access",
        url::form_urlencoded::byte_serialize(state.oauth_state.as_bytes()).collect::<String>(),
        state.challenge
    );

    println!(
        "Open the following link in your browser:

{}
",
        oauth_uri
    );

    let sock_addr = SocketAddr::new(IpAddr::V4(Ipv4Addr::LOCALHOST), 6009);
    let app = Router::new()
        .route("/", get(receive_token))
        .layer(Extension(state));

    axum::Server::bind(&sock_addr)
        .serve(app.into_make_service())
        .with_graceful_shutdown(async { rx.recv().await.unwrap() })
        .await
        .expect("Unable to start server");

    Ok(())
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

    state.shutdown_signal.send(()).await.unwrap();

    Html("<h1>Hello from twitter-auth</h1><p>Your tokens are displayed on the standard output.</p>")
}
