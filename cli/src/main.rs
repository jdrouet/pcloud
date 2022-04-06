mod config;
mod file;
mod folder;

use clap::Parser;
use pcloud::http::HttpClient;
use std::path::PathBuf;

#[derive(Parser)]
#[clap(about, author, version)]
struct Command {
    /// Path to load the configuration file. Default to ~/.config/pcloud.json. If not found, loading from environment.
    #[clap(short, long)]
    config: Option<PathBuf>,
    #[clap(short, long)]
    verbose: bool,
    #[clap(subcommand)]
    subcmd: SubCommand,
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
}

#[derive(Parser)]
enum SubCommand {
    /// Folder related sub command
    #[clap()]
    Folder(folder::Command),
    /// File related sub command
    #[clap()]
    File(file::Command),
}

impl Command {
    async fn execute(&self, pcloud: HttpClient) {
        match &self.subcmd {
            SubCommand::Folder(sub) => sub.execute(pcloud).await,
            SubCommand::File(sub) => sub.execute(pcloud).await,
        }
    }

    fn set_log_level(&self) {
        if self.verbose {
            tracing_subscriber::fmt()
                .with_env_filter("info")
                .try_init()
                .expect("couldn't init logger");
        }
    }
}

#[tokio::main]
async fn main() {
    let cmd = Command::parse();
    cmd.set_log_level();
    let cfg = config::Config::from_path(&cmd.config()).unwrap_or_default();
    let pcloud = cfg.build().expect("couldn't build client");
    cmd.execute(pcloud).await;
}
