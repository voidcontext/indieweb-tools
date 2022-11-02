use std::fs;

use serde_derive::Deserialize;
use simple_logger::SimpleLogger;

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
    /// Print debug information, including secrets!
    #[clap(short, long, action)]
    debug: bool,
    /// Path to the config file
    #[clap(long, value_parser, default_value_t = String::from("config.toml"))]
    config: String,
    /// Update auth tokens in the given sqlite DB
    #[clap(long, value_parser)]
    db_path: Option<String>,
}

#[derive(Subcommand)]
enum Commands {
    /// Start an Oauth 2.0 app authentication flow
    Auth {
        #[clap(subcommand)]
        social_network: AuthSubcommands,
    },
}

#[derive(Subcommand)]
enum AuthSubcommands {
    /// Twitter Oauth flow
    Twitter,
    /// Mastodon Oauth flow
    Mastodon,
}

// Clap - end

#[derive(Debug, Deserialize)]
pub struct Config {
    twitter: TwitterConfig,
}

#[derive(Debug, Deserialize)]
pub struct TwitterConfig {
    client_id: String,
}

impl Config {
    pub fn from_file(file_name: &str) -> Result<Config, toml::de::Error> {
        let config_str = fs::read_to_string(file_name)
            .unwrap_or_else(|_| panic!("The file '{}' doesn't exist", file_name));

        toml::from_str(&config_str)
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cli = Cli::parse();

    // Load config
    let config = Config::from_file(&cli.config)?;

    // Initialise logger
    let log_level = if cli.debug { Debug } else { Info };
    SimpleLogger::new().with_level(log_level).init().unwrap();

    match cli.command {
        Commands::Auth { social_network } => match social_network {
            AuthSubcommands::Twitter => twitter::start_flow(&config, cli.db_path)
                .await
                .map_err(|e| Box::new(e) as Box<dyn std::error::Error>),
            AuthSubcommands::Mastodon => todo!(),
        },
    }
}
