use clap::Parser;
use pcloud::http::HttpClient;
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
        let params = pcloud::file::upload::Params::new(filename.as_str(), self.folder_id)
            .no_partial(!self.allow_partial_upload);
        match pcloud.upload_file(&file, &params).await {
            Ok(res) => {
                tracing::info!("file uploaded: {}", res.file_id);
                std::process::exit(exitcode::OK);
            }
            Err(err) => {
                tracing::error!("unable to upload file: {:?}", err);
                std::process::exit(exitcode::DATAERR);
            }
        }
    }
}
