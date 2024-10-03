mod cmd;
mod config;

#[cfg(all(test, feature = "protected"))]
mod tests;

use clap::Parser;
use pcloud::client::HttpClient;
use std::path::PathBuf;

#[derive(Parser)]
#[clap(about, author, version)]
struct Command {
    /// Path to load the configuration file. Default to ~/.config/pcloud.json. If not found, loading from environment.
    #[clap(short, long)]
    config: Option<PathBuf>,
    /// Turns on debug information
    #[arg(short, long, action = clap::ArgAction::Count)]
    verbose: u8,
    #[clap(subcommand)]
    subcmd: cmd::Command,
}

impl Command {
    fn config(&self) -> PathBuf {
        if let Some(ref cfg) = self.config {
            cfg.clone()
        } else if let Some(cfg_dir) = dirs::config_dir() {
            cfg_dir.join("pcloud.json")
        } else {
            PathBuf::from(".pcloud.json")
        }
    }

    fn log_level(&self) -> &str {
        match self.verbose {
            0 => "danger",
            1 => "warn",
            2 => "info",
            3 => "debug",
            _ => "trace",
        }
    }
}

impl Command {
    async fn execute(self, client: HttpClient) -> anyhow::Result<()> {
        self.subcmd.execute(&client).await
    }

    fn set_log_level(&self) {
        tracing_subscriber::fmt()
            .with_env_filter(self.log_level())
            .try_init()
            .expect("couldn't init logger");
    }
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let cmd = Command::parse();
    cmd.set_log_level();
    let cfg = config::Config::from_path(&cmd.config()).unwrap_or_default();
    let pcloud = cfg.build().expect("couldn't build client");
    cmd.execute(pcloud).await
}
