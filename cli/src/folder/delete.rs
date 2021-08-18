use clap::Clap;
use pcloud::http::PCloudHttpApi;

#[derive(Clap)]
pub struct Command {
    #[clap(short, long)]
    recursive: bool,
}

impl Command {
    pub async fn execute(&self, pcloud: PCloudHttpApi, folder_id: usize) {
        let result = if self.recursive {
            pcloud
                .delete_folder_recursive(folder_id)
                .await
                .map(|_| log::info!("folder {} deleted", folder_id))
        } else {
            pcloud
                .delete_folder(folder_id)
                .await
                .map(|_| log::info!("folder {} deleted", folder_id))
        };
        match result {
            Ok(_) => {
                std::process::exit(exitcode::OK);
            }
            Err(err) => {
                log::error!("unable to delete folder: {:?}", err);
                std::process::exit(exitcode::DATAERR);
            }
        }
    }
}
