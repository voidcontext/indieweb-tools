use std::fs;

use serde_derive::Deserialize;
use simple_logger::SimpleLogger;
use tokio::sync::mpsc::Sender;

use log::LevelFilter::{Debug, Info};

use clap::{Parser, Subcommand};

mod twitter;

// Clap

#[derive(Parser)]
#[clap(author, version, about, long_about = None)]
#[clap(propagate_version = true)]
struct Cli {
    #[clap(subcommand)]
    command: Commands,
    /// Print debug information, incliding secrets!
    #[clap(short, long, action)]
    debug: bool,
}

#[derive(Subcommand)]
enum Commands {
    /// Start an Oauth 2.0 app authentication flow
    Auth {
        #[clap(subcommand)]
        provider: AuthSubcommands,
    },
}

#[derive(Subcommand)]
enum AuthSubcommands {
    Twitter,
    Mastodon,
}

// Clap - end

#[derive(Debug, Deserialize)]
pub struct Config {
    client_id: String,
    db: DBConfig,
}

#[derive(Debug, Deserialize)]
struct DBConfig {
    path: String,
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
    db: sled::Db,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cli = Cli::parse();

    // Load config
    let config = Config::from_file("config.toml")?;

    // Initialise logger
    let log_level = if cli.debug { Debug } else { Info };
    SimpleLogger::new().with_level(log_level).init().unwrap();

    match cli.command {
        Commands::Auth { provider } => match provider {
            AuthSubcommands::Twitter => twitter::start_flow(&config)
                .await
                .map_err(|e| Box::new(e) as Box<dyn std::error::Error>),
            AuthSubcommands::Mastodon => todo!(),
        },
    }
}
