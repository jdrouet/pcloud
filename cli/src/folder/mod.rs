mod common;

mod create;
mod delete;
mod download;
mod list;
mod upload;

use clap::Parser;
use pcloud::client::HttpClient;

#[derive(Parser)]
pub struct Command {
    #[clap(default_value = "0")]
    folder_id: u64,
    #[clap(subcommand)]
    subcommand: SubCommand,
}

impl Command {
    pub async fn execute(&self, pcloud: HttpClient) {
        self.subcommand.execute(pcloud, self.folder_id).await
    }
}

#[derive(Parser)]
enum SubCommand {
    Create(create::Command),
    Delete(delete::Command),
    Download(download::Command),
    List(list::Command),
    Upload(upload::Command),
}

impl SubCommand {
    pub async fn execute(&self, pcloud: HttpClient, folder_id: u64) {
        match self {
            Self::Create(cmd) => cmd.execute(pcloud, folder_id).await,
            Self::Delete(cmd) => cmd.execute(pcloud, folder_id).await,
            Self::Download(cmd) => cmd.execute(pcloud, folder_id).await,
            Self::List(cmd) => cmd.execute(pcloud, folder_id).await,
            Self::Upload(cmd) => cmd.execute(pcloud, folder_id).await,
        }
    }
}
