mod delete;
mod download;
mod moving;
mod rename;
mod upload;

use clap::Parser;
use pcloud::http::HttpClient;

#[derive(Parser)]
pub struct Command {
    #[clap(subcommand)]
    subcommand: SubCommand,
}

impl Command {
    pub async fn execute(&self, pcloud: HttpClient) {
        self.subcommand.execute(pcloud).await
    }
}

#[derive(Parser)]
enum SubCommand {
    Delete(delete::Command),
    Download(download::Command),
    Move(moving::Command),
    Rename(rename::Command),
    Upload(upload::Command),
}

impl SubCommand {
    pub async fn execute(&self, pcloud: HttpClient) {
        match self {
            Self::Delete(cmd) => cmd.execute(pcloud).await,
            Self::Download(cmd) => cmd.execute(pcloud).await,
            Self::Move(cmd) => cmd.execute(pcloud).await,
            Self::Rename(cmd) => cmd.execute(pcloud).await,
            Self::Upload(cmd) => cmd.execute(pcloud).await,
        }
    }
}
