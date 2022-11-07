use clap::Parser;
use clap::Subcommand;

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
    AppAuth {
        #[clap(subcommand)]
        sub_command: iwt_app_auth::AuthSubcommand,
    },
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cli = Cli::parse();

    let config = Config::from_file(&cli.config)?;

    match cli.command {
        Command::AppAuth { sub_command } => iwt_app_auth::execute(sub_command, &config),
    }
    .await
}
