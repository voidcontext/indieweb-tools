use std::{env, fmt::Display};

pub use crate::rss::*;
pub use config::Config;
use log::LevelFilter::{Debug, Info};
use simple_logger::SimpleLogger;

use crate::syndicate::Target;

mod config;
mod rss;
mod syndicate;

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
    println!("Hello, world!");

    let log_level =
        env::var("ORION_DEBUG").map_or(Info, |debug| if debug == "1" { Debug } else { Info });
    SimpleLogger::new().with_level(log_level).init().unwrap();

    let config_file = env::var("ORION_CONFIG_FILE").expect("Env var ORION_CONFIG_FILE must be set");
    let targets: Vec<Box<dyn Target>> = vec![];

    let result = match Config::from_file(&config_file) {
        Err(err) => {
            log::error!("Couldn't load config: {:?}", err);
            Result::<(), Box<dyn std::error::Error>>::Err(Box::new(OrionError::ConfigError))
        }
        Ok(config) => {
            log::debug!("Config loaded");
            syndicate::syndicate(&config, Box::new(RssClientImpl), &targets).await
        }
    };

    log::debug!("Main result is {:?}", result);

    result
}

#[cfg(test)]
pub mod stubs {

    pub use crate::rss::stubs as rss;
}
