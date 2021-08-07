mod delete;
mod download;
mod upload;

use clap::Clap;
use pcloud::PCloudApi;

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
    Upload(upload::Command),
}

impl SubCommand {
    pub async fn execute(&self, pcloud: PCloudApi) {
        match self {
            Self::Delete(cmd) => cmd.execute(pcloud).await,
            Self::Download(cmd) => cmd.execute(pcloud).await,
            Self::Upload(cmd) => cmd.execute(pcloud).await,
        }
    }
}
