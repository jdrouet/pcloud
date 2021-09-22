use clap::Clap;
use pcloud::file::rename::Params;
use pcloud::http::HttpClient;

#[derive(Clap)]
pub struct Command {
    file_id: usize,
    folder_id: usize,
}

impl Command {
    #[tracing::instrument(skip_all)]
    pub async fn execute(&self, pcloud: HttpClient) {
        let params = Params::new_move(self.file_id, self.folder_id);
        match pcloud.rename_file(&params).await {
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
