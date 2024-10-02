use clap::Parser;
use pcloud::client::HttpClient;
use pcloud::file::rename::FileMoveCommand;
use pcloud::prelude::HttpCommand;

#[derive(Parser)]
pub struct Command {
    file_id: u64,
    folder_id: u64,
}

impl Command {
    #[tracing::instrument(skip_all)]
    pub async fn execute(&self, pcloud: HttpClient) {
        match FileMoveCommand::new(self.file_id.into(), self.folder_id.into())
            .execute(&pcloud)
            .await
        {
            Ok(_) => {
                tracing::info!("file moved");
                std::process::exit(exitcode::OK);
            }
            Err(err) => {
                tracing::error!("unable to move file: {:?}", err);
                std::process::exit(exitcode::DATAERR);
            }
        }
    }
}
