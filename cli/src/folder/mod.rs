mod create;
mod delete;
mod list;
mod sync;

use clap::Clap;
use pcloud::http::HttpClient;

#[derive(Clap)]
pub struct Command {
    #[clap(default_value = "0")]
    folder_id: usize,
    #[clap(subcommand)]
    subcommand: SubCommand,
}

impl Command {
    pub async fn execute(&self, pcloud: HttpClient) {
        self.subcommand.execute(pcloud, self.folder_id).await
    }
}

#[derive(Clap)]
enum SubCommand {
    Create(create::Command),
    Delete(delete::Command),
    List(list::Command),
    Sync(sync::Command),
}

impl SubCommand {
    pub async fn execute(&self, pcloud: HttpClient, folder_id: usize) {
        match self {
            Self::Create(cmd) => cmd.execute(pcloud, folder_id).await,
            Self::Delete(cmd) => cmd.execute(pcloud, folder_id).await,
            Self::List(cmd) => cmd.execute(pcloud, folder_id).await,
            Self::Sync(cmd) => cmd.execute(pcloud, folder_id).await,
        }
    }
}
