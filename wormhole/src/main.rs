use std::{net::{SocketAddr, IpAddr, Ipv4Addr}, env, rc::Rc, sync::Arc};

use axum::{Router, Extension, routing::put, response::IntoResponse, extract::Path};
use rand::{thread_rng, distributions::Alphanumeric, Rng};
use tokio_rusqlite::Connection;

#[derive(Clone)]
struct State {
    db_conn: Arc<Connection>
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("Hello, world!");
    
    let db_path = env::var("WORMHOLE_DB_PATH").expect("WORMHOLE_DB_PATH must be set.");
    let db_conn = Connection::open(db_path).await.unwrap();
    db_conn.call(|conn| conn.execute(
        "
        CREATE TABLE IF NOT EXISTS permashortlink (
            url   TEXT PRIMARY KEY,
            short VARCHAR(5)
        )
        ",
        ()
    )).await.unwrap();
    
    let state = State {db_conn: Arc::new(db_conn)};
    
    let sock_addr = SocketAddr::new(IpAddr::V4(Ipv4Addr::LOCALHOST), 6009);
    let app = Router::new()
        .route("/s/:url", put(add_url))
        // shate the state with the request handler
        .layer(Extension(state));

    axum::Server::bind(&sock_addr)
        .serve(app.into_make_service())
        .await
        .map_err(|e| Box::new(e) as Box::<dyn std::error::Error>)
}

// TODO: get conn from state
async fn add_url(Path(url): Path<String>) -> impl IntoResponse {
    let mut short = gen_short();
    
    while Some(_) = find_short(short, conn) {
        short = gen_short();
    }
}

fn gen_short() -> String {
    thread_rng()
        .sample_iter(&Alphanumeric)
        .take(4)
        .map(char::from)
        .collect()
}

fn find_short(short: &String, conn: rusqlite::Connection) -> rusqlite::Result<Option<String>> {
    let mut statement = conn.prepare("SELECT short FROM permashortlink WHERE short = :short")?;
    
    statement.query_row(&[(":short", short.as_str())], |row| row.get(0))
}

fn find_url(url: &String, conn: rusqlite::Connection) -> rusqlite::Result<Option<String>> {
    let mut statement = conn.prepare("SELECT short FROM permashortlink WHERE url = :url")?;
    
    statement.query_row(&[(":url", url.as_str())], |row| row.get(0))
}