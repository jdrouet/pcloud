use clap::Clap;
use pcloud::http::PCloudApi;

#[derive(Clap)]
pub struct Command {
    file_id: usize,
    filename: String,
}

impl Command {
    pub async fn execute(&self, pcloud: PCloudApi) {
        match pcloud
            .rename_file(self.file_id, self.filename.as_str())
            .await
        {
            Ok(_) => {
                log::info!("file renamed");
                std::process::exit(exitcode::OK);
            }
            Err(err) => {
                log::error!("unable to rename file: {:?}", err);
                std::process::exit(exitcode::DATAERR);
            }
        }
    }
}
