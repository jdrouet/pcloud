mod create;
mod delete;
mod list;

use clap::Clap;
use pcloud::PCloudApi;

#[derive(Clap)]
pub struct Command {
    #[clap(default_value = "0")]
    folder_id: usize,
    #[clap(subcommand)]
    subcommand: SubCommand,
}

impl Command {
    pub async fn execute(&self, pcloud: PCloudApi) {
        self.subcommand.execute(pcloud, self.folder_id).await
    }
}

#[derive(Clap)]
enum SubCommand {
    Create(create::Command),
    Delete(delete::Command),
    List(list::Command),
}

impl SubCommand {
    pub async fn execute(&self, pcloud: PCloudApi, folder_id: usize) {
        match self {
            Self::Create(cmd) => cmd.execute(pcloud, folder_id).await,
            Self::Delete(cmd) => cmd.execute(pcloud, folder_id).await,
            Self::List(cmd) => cmd.execute(pcloud, folder_id).await,
        }
    }
}
