use clap::Parser;
use pcloud::file::upload::MultipartFileUploadCommand;
use pcloud::http::HttpClient;
use pcloud::prelude::HttpCommand;
use std::fs::File;
use std::path::PathBuf;

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
        let file = File::open(&self.path).expect("unable to open file");
        let filename = self.filename();
        let cmd = MultipartFileUploadCommand::new(self.folder_id);
        let cmd = match cmd.add_sync_entry(filename, file) {
            Ok(cmd) => cmd,
            Err(err) => {
                tracing::error!("unable to read file: {:?}", err);
                std::process::exit(exitcode::DATAERR);
            }
        };
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
