use clap::Parser;
use pcloud::http::HttpClient;
use std::fs::OpenOptions;
use std::path::PathBuf;

#[derive(Parser)]
pub struct Command {
    /// Overrides an existing file
    #[clap(long)]
    overrides: bool,
    /// Remote file id
    file_id: u64,
    /// Output path for the file
    path: PathBuf,
}

impl Command {
    #[tracing::instrument(skip_all)]
    pub async fn execute(&self, pcloud: HttpClient) {
        let file = OpenOptions::new()
            .create_new(!self.overrides)
            .create(true)
            .truncate(true)
            .open(&self.path)
            .expect("unable to create file");
        match pcloud.download_file(self.file_id, file).await {
            Ok(res) => {
                tracing::info!("file downloaded: {}", res);
                std::process::exit(exitcode::OK);
            }
            Err(err) => {
                tracing::error!("unable to upload file: {:?}", err);
                std::process::exit(exitcode::DATAERR);
            }
        }
    }
}
