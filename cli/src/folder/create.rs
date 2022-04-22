use clap::Parser;
use pcloud::folder::create::FolderCreateCommand;
use pcloud::http::HttpClient;
use pcloud::prelude::HttpCommand;

#[derive(Parser)]
pub struct Command {
    name: String,
}

impl Command {
    #[tracing::instrument(skip_all)]
    pub async fn execute(&self, pcloud: HttpClient, folder_id: u64) {
        match FolderCreateCommand::new(self.name.clone(), folder_id)
            .execute(&pcloud)
            .await
        {
            Ok(res) => {
                tracing::info!("folder created {}", res.folder_id);
                std::process::exit(exitcode::OK);
            }
            Err(err) => {
                tracing::error!("unable to create folder: {:?}", err);
                std::process::exit(exitcode::CANTCREAT);
            }
        }
    }
}
