use clap::Clap;
use pcloud::folder::create::Params;
use pcloud::http::PCloudApi;

#[derive(Clap)]
pub struct Command {
    name: String,
}

impl Command {
    pub async fn execute(&self, pcloud: PCloudApi, folder_id: usize) {
        let params = Params::new(&self.name, folder_id);
        match pcloud.create_folder(&params).await {
            Ok(res) => {
                log::info!("folder created {}", res.folder_id);
                std::process::exit(exitcode::OK);
            }
            Err(err) => {
                log::error!("unable to create folder: {:?}", err);
                std::process::exit(exitcode::CANTCREAT);
            }
        }
    }
}
