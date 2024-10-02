use clap::Parser;
use pcloud::client::HttpClient;
use pcloud::file::rename::FileRenameCommand;
use pcloud::prelude::HttpCommand;

#[derive(Parser)]
pub struct Command {
    file_id: u64,
    filename: String,
}

impl Command {
    #[tracing::instrument(skip_all)]
    pub async fn execute(&self, pcloud: HttpClient) {
        match FileRenameCommand::new(self.file_id.into(), self.filename.clone())
            .execute(&pcloud)
            .await
        {
            Ok(_) => {
                tracing::info!("file renamed");
                std::process::exit(exitcode::OK);
            }
            Err(err) => {
                tracing::error!("unable to rename file: {:?}", err);
                std::process::exit(exitcode::DATAERR);
            }
        }
    }
}
