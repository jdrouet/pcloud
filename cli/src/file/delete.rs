use clap::Clap;
use pcloud::http::HttpClient;

#[derive(Clap)]
pub struct Command {
    file_id: usize,
}

impl Command {
    #[tracing::instrument(skip_all)]
    pub async fn execute(&self, pcloud: HttpClient) {
        match pcloud.delete_file(self.file_id).await {
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
