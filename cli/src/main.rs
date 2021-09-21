mod config;
mod file;
mod folder;

use clap::Clap;
use pcloud::http::HttpClient;
use std::path::PathBuf;

#[derive(Clap)]
struct Command {
    #[clap(short, long)]
    config: Option<PathBuf>,
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

#[derive(Clap)]
enum SubCommand {
    Folder(folder::Command),
    File(file::Command),
}

impl Command {
    async fn execute(&self, pcloud: HttpClient) {
        match &self.subcmd {
            SubCommand::Folder(sub) => sub.execute(pcloud).await,
            SubCommand::File(sub) => sub.execute(pcloud).await,
        }
    }
}

#[tokio::main]
async fn main() {
    env_logger::init();

    let cmd = Command::parse();
    let pcloud = config::Config::from_path(&cmd.config())
        .map(|cfg| cfg.build())
        .unwrap_or_else(|_| HttpClient::from_env());
    cmd.execute(pcloud).await;
}
