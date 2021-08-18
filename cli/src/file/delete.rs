use clap::Clap;
use pcloud::http::PCloudHttpApi;

#[derive(Clap)]
pub struct Command {
    file_id: usize,
}

impl Command {
    pub async fn execute(&self, pcloud: PCloudHttpApi) {
        match pcloud.delete_file(self.file_id).await {
            Ok(_) => {
                log::info!("file deleted");
                std::process::exit(exitcode::OK);
            }
            Err(err) => {
                log::error!("unable to delete file: {:?}", err);
                std::process::exit(exitcode::DATAERR);
            }
        }
    }
}
