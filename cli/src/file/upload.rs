use clap::Clap;
use pcloud::http::HttpClient;
use std::fs::File;
use std::path::PathBuf;

#[derive(Clap)]
pub struct Command {
    #[clap(long)]
    filename: Option<String>,
    #[clap(long, default_value = "0")]
    folder_id: usize,
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
        match pcloud.upload_file(&file, &filename, self.folder_id).await {
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
