use clap::Clap;
use pcloud::http::PCloudApi;
use std::fs::File;
use std::path::PathBuf;

#[derive(Clap)]
pub struct Command {
    file_id: usize,
    path: PathBuf,
}

impl Command {
    pub async fn execute(&self, pcloud: PCloudApi) {
        let file = File::create(&self.path).expect("unable to create file");
        match pcloud.download_file(self.file_id, file).await {
            Ok(res) => {
                log::info!("file downloaded: {}", res);
                std::process::exit(exitcode::OK);
            }
            Err(err) => {
                log::error!("unable to upload file: {:?}", err);
                std::process::exit(exitcode::DATAERR);
            }
        }
    }
}
