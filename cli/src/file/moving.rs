use clap::Clap;
use pcloud::file::rename::Params;
use pcloud::http::PCloudApi;

#[derive(Clap)]
pub struct Command {
    file_id: usize,
    folder_id: usize,
}

impl Command {
    pub async fn execute(&self, pcloud: PCloudApi) {
        let params = Params::new_move(self.file_id, self.folder_id);
        match pcloud.rename_file(&params).await {
            Ok(_) => {
                log::info!("file moved");
                std::process::exit(exitcode::OK);
            }
            Err(err) => {
                log::error!("unable to move file: {:?}", err);
                std::process::exit(exitcode::DATAERR);
            }
        }
    }
}
