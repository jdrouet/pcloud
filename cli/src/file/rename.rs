use clap::Clap;
use pcloud::file::rename::Params;
use pcloud::http::PCloudHttpApi;

#[derive(Clap)]
pub struct Command {
    file_id: usize,
    filename: String,
}

impl Command {
    pub async fn execute(&self, pcloud: PCloudHttpApi) {
        let params = Params::new_rename(self.file_id, self.filename.clone());
        match pcloud.rename_file(&params).await {
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
