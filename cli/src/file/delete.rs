use clap::Parser;
use pcloud::client::HttpClient;
use pcloud::file::delete::FileDeleteCommand;
use pcloud::prelude::HttpCommand;

#[derive(Parser)]
pub struct Command {
    file_id: u64,
}

impl Command {
    #[tracing::instrument(skip_all)]
    pub async fn execute(&self, pcloud: HttpClient) {
        match FileDeleteCommand::new(self.file_id.into())
            .execute(&pcloud)
            .await
        {
            Ok(_) => {
                tracing::info!("file deleted");
                std::process::exit(exitcode::OK);
            }
            Err(err) => {
                tracing::error!("unable to delete file: {:?}", err);
                std::process::exit(exitcode::DATAERR);
            }
        }
    }
}
