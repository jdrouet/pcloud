mod delete;
mod download;
mod moving;
mod rename;
mod upload;

use clap::Clap;
use pcloud::http::PCloudApi;

#[derive(Clap)]
pub struct Command {
    #[clap(subcommand)]
    subcommand: SubCommand,
}

impl Command {
    pub async fn execute(&self, pcloud: PCloudApi) {
        self.subcommand.execute(pcloud).await
    }
}

#[derive(Clap)]
enum SubCommand {
    Delete(delete::Command),
    Download(download::Command),
    Move(moving::Command),
    Rename(rename::Command),
    Upload(upload::Command),
}

impl SubCommand {
    pub async fn execute(&self, pcloud: PCloudApi) {
        match self {
            Self::Delete(cmd) => cmd.execute(pcloud).await,
            Self::Download(cmd) => cmd.execute(pcloud).await,
            Self::Move(cmd) => cmd.execute(pcloud).await,
            Self::Rename(cmd) => cmd.execute(pcloud).await,
            Self::Upload(cmd) => cmd.execute(pcloud).await,
        }
    }
}
