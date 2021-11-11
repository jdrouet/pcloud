use clap::Parser;
use pcloud::http::HttpClient;
use std::fs::File;
use std::path::PathBuf;

#[derive(Parser)]
pub struct Command {
    file_id: usize,
    path: PathBuf,
}

impl Command {
    #[tracing::instrument(skip_all)]
    pub async fn execute(&self, pcloud: HttpClient) {
        let file = File::create(&self.path).expect("unable to create file");
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
