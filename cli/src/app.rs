use clap::Parser;
use pcloud::http::HttpClient;
use std::path::PathBuf;

#[derive(Parser)]
#[clap(about, author, version)]
pub struct Command {
    /// Path to load the configuration file. Default to ~/.config/pcloud.json. If not found, loading from environment.
    #[clap(short, long)]
    config: Option<PathBuf>,
    #[clap(short, long)]
    verbose: bool,
    #[clap(subcommand)]
    subcmd: SubCommand,
}

impl Command {
    pub fn config(&self) -> PathBuf {
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
pub enum SubCommand {
    /// Folder related sub command
    #[clap()]
    Folder(crate::folder::Command),
    /// File related sub command
    #[clap()]
    File(crate::file::Command),
}

impl Command {
    pub async fn execute(&self, pcloud: HttpClient) {
        match &self.subcmd {
            SubCommand::Folder(sub) => sub.execute(pcloud).await,
            SubCommand::File(sub) => sub.execute(pcloud).await,
        }
    }

    pub fn set_log_level(&self) {
        if self.verbose {
            tracing_subscriber::fmt()
                .with_env_filter("info")
                .try_init()
                .expect("couldn't init logger");
        }
    }
}
