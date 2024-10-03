use clap::Parser;
use pcloud::client::HttpClient;
use pcloud::file::upload::MultipartFileUploadCommand;
use pcloud::prelude::HttpCommand;
use std::path::PathBuf;
use tokio::fs::File;

#[derive(Parser)]
pub struct Command {
    /// Name of the created remote file.
    #[clap(long)]
    filename: Option<String>,
    /// Folder to store the file in.
    #[clap(long, default_value = "0")]
    folder_id: u64,
    /// Keep partial file if upload fails.
    #[clap(long)]
    allow_partial_upload: bool,
    path: PathBuf,
}

impl Command {
    fn filename(&self) -> String {
        self.filename
            .clone()
            .or_else(|| {
                self.path
                    .file_name()
                    .and_then(|value| value.to_str())
                    .map(|value| value.to_string())
            })
            .unwrap()
    }

    #[tracing::instrument(skip_all)]
    pub async fn execute(&self, pcloud: HttpClient) {
        let fsize = std::fs::metadata(&self.path).expect("unable to read file size");
        let file = File::open(&self.path).await.expect("unable to open file");
        let filename = self.filename();
        let mut cmd = MultipartFileUploadCommand::new(self.folder_id);
        cmd.add_body_entry(filename, fsize.len(), file);
        match cmd.execute(&pcloud).await {
            Ok(res) => {
                tracing::info!(
                    "file uploaded: {:?}",
                    res.iter().map(|f| f.file_id).collect::<Vec<_>>()
                );
                std::process::exit(exitcode::OK);
            }
            Err(err) => {
                tracing::error!("unable to upload file: {:?}", err);
                std::process::exit(exitcode::DATAERR);
            }
        }
    }
}
