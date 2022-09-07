use std::{fmt::Display, rc::Rc};

pub use crate::rss::*;
use crate::{
    auth::token_db::SqliteTokenDB, mastodon::Mastodon,
    syndicated_post::SqliteSyndycatedPostStorage, twitter::Twitter,
};
use clap::Parser;
pub use config::Config;
use log::LevelFilter::{Debug, Info};
use rusqlite::Connection;
use simple_logger::SimpleLogger;

pub use crate::target::Target;

mod auth;
mod config;
mod mastodon;
mod provider;
mod rss;
mod syndicate;
mod syndicated_post;
mod target;
mod twitter;

#[derive(Parser)]
#[clap(author, version, about, long_about = None)]
#[clap(propagate_version = true)]
struct Cli {
    /// Print debug information, including secrets!
    #[clap(short, long, action)]
    debug: bool,
    /// Path to the config file
    #[clap(long, value_parser, default_value_t = String::from("config.toml"))]
    config: String,
}

#[derive(Debug)]
enum OrionError {
    ConfigError,
}

impl Display for OrionError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                OrionError::ConfigError => "OrionError::ConfigError",
            }
        )
    }
}

impl std::error::Error for OrionError {}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cli = Cli::parse();

    let log_level = if cli.debug { Debug } else { Info };
    SimpleLogger::new().with_level(log_level).init().unwrap();

    let result = match Config::from_file(&cli.config) {
        Err(err) => {
            log::error!("Couldn't load config: {:?}", err);
            Result::<(), Box<dyn std::error::Error>>::Err(Box::new(OrionError::ConfigError))
        }
        Ok(config) => {
            log::debug!("Config loaded");
            let conn = Rc::new(Connection::open(&config.db.path).expect("Couldn't open DB"));

            let token_db = SqliteTokenDB::new(Rc::clone(&conn));

            let targets: Vec<Box<dyn Target>> = vec![
                Box::new(Twitter::new(config.twitter.client_id.clone(), token_db)),
                Box::new(Mastodon::new(
                    config.mastodon.base_uri.clone(),
                    config.mastodon.access_token.clone(),
                )),
            ];

            let storage = SqliteSyndycatedPostStorage::new(Rc::clone(&conn));
            storage
                .init_table()
                .expect("Couldn't initialise post storage");

            syndicate::syndicate(&config, &RssClientImpl, &targets, &storage).await
        }
    };

    log::debug!("Main result is {:?}", result);

    result
}

#[cfg(test)]
pub mod stubs {
    pub use crate::auth::token_db::stubs as token_db;
    pub use crate::rss::stubs as rss;
    pub use crate::syndicated_post::stubs as syndycated_post;
    pub use crate::target::stubs as target;
}
