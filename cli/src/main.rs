mod config;
mod file;
mod folder;

use clap::{crate_authors, crate_description, crate_version, Parser};
use pcloud::http::{reqwest, HttpClient};
use std::path::PathBuf;
use std::time::Duration;

#[derive(Parser)]
#[clap(about = crate_description!(), author = crate_authors!(), version = crate_version!())]
struct Command {
    #[clap(
        short,
        long,
        about = "Path to load the configuration file. Default to ~/.config/pcloud.json. If not found, loading from environment."
    )]
    config: Option<PathBuf>,
    #[clap(long, about = "Connection timeout in seconds.", default_value = "1.0")]
    connect_timeout: f64,
    #[clap(long, about = "Request timeout in seconds.", default_value = "10.0")]
    request_timeout: f64,
    #[clap(short, long)]
    verbose: bool,
    #[clap(subcommand)]
    subcmd: SubCommand,
}

impl Command {
    fn client(&self) -> reqwest::Client {
        HttpClient::client_builder()
            .connect_timeout(Duration::from_secs_f64(self.connect_timeout))
            .timeout(Duration::from_secs_f64(self.request_timeout))
            .build()
            .expect("couldn't build http client")
    }

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
    #[clap(about = "Folder related sub command")]
    Folder(folder::Command),
    #[clap(about = "File related sub command")]
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
    let pcloud = config::Config::from_path(&cmd.config())
        .map(|cfg| cfg.build())
        .unwrap_or_else(|_| HttpClient::from_env())
        .with_client(cmd.client());
    cmd.execute(pcloud).await;
}
