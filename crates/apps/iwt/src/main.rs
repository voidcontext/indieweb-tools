use clap::Parser;
use clap::Subcommand;

use log::LevelFilter::{Debug, Info};
use simple_logger::SimpleLogger;

use iwt_config::Config;

#[derive(Parser)]
#[clap(author, version, about, long_about = None)]
#[clap(propagate_version = true)]
struct Cli {
    #[clap(subcommand)]
    command: Command,
    /// Print debug information, including secrets!
    #[clap(short, long, action)]
    debug: bool,
    /// Path to the config file
    #[clap(long, value_parser, default_value_t = String::from("config.toml"))]
    config: String,
}

#[derive(Subcommand)]
enum Command {
    /// App Authentication helper
    AppAuth {
        #[clap(subcommand)]
        sub_command: iwt_app_auth::AuthSubcommand,
    },
    /// Cross publish posts
    CrossPublish,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cli = Cli::parse();

    let log_level = if cli.debug { Debug } else { Info };
    SimpleLogger::new().with_level(log_level).init().unwrap();

    let config = Config::from_file(&cli.config)?;

    match cli.command {
        Command::AppAuth { sub_command } => iwt_app_auth::execute(sub_command, &config).await,
        Command::CrossPublish => iwt_cross_publisher::execute(&config).await,
    }
}
