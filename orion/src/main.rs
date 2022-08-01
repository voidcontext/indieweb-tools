use std::{env, fmt::Display};

pub use crate::rss::*;
use crate::{auth::token_db::SledTokenDB, twitter::Twitter};
pub use config::Config;
use log::LevelFilter::{Debug, Info};
use simple_logger::SimpleLogger;

pub use crate::target::Target;

mod auth;
mod config;
mod mastodon;
mod rss;
mod syndicate;
mod target;
mod twitter;

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
    let log_level =
        env::var("ORION_DEBUG").map_or(Info, |debug| if debug == "1" { Debug } else { Info });
    SimpleLogger::new().with_level(log_level).init().unwrap();

    let config_file = env::var("ORION_CONFIG_FILE").expect("Env var ORION_CONFIG_FILE must be set");

    let result = match Config::from_file(&config_file) {
        Err(err) => {
            log::error!("Couldn't load config: {:?}", err);
            Result::<(), Box<dyn std::error::Error>>::Err(Box::new(OrionError::ConfigError))
        }
        Ok(config) => {
            log::debug!("Config loaded");

            let token_db = SledTokenDB::new(&config.db.path);

            let targets: Vec<Box<dyn Target>> = vec![Box::new(Twitter::new(
                config.twitter.client_id.clone(),
                token_db,
            ))];

            syndicate::syndicate(&config, Box::new(RssClientImpl), &targets).await
        }
    };

    log::debug!("Main result is {:?}", result);

    result
}

#[cfg(test)]
pub mod stubs {
    pub use crate::auth::token_db::stubs as token_db;
    pub use crate::rss::stubs as rss;
    pub use crate::target::stubs as target;
}
