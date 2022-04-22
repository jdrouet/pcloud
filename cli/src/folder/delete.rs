use clap::Parser;
use pcloud::folder::delete::FolderDeleteCommand;
use pcloud::http::HttpClient;
use pcloud::prelude::HttpCommand;

#[derive(Parser)]
pub struct Command {
    #[clap(short, long)]
    recursive: bool,
}

impl Command {
    #[tracing::instrument(skip_all)]
    pub async fn execute(&self, pcloud: HttpClient, folder_id: u64) {
        match FolderDeleteCommand::new(folder_id.into())
            .recursive(self.recursive)
            .execute(&pcloud)
            .await
        {
            Ok(_) => {
                tracing::info!("folder deleted");
                std::process::exit(exitcode::OK);
            }
            Err(err) => {
                tracing::error!("unable to delete folder: {:?}", err);
                std::process::exit(exitcode::DATAERR);
            }
        }
    }
}
