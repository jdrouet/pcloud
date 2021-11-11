use clap::Parser;
use pcloud::http::HttpClient;

#[derive(Parser)]
pub struct Command {
    #[clap(short, long)]
    recursive: bool,
}

impl Command {
    #[tracing::instrument(skip_all)]
    pub async fn execute(&self, pcloud: HttpClient, folder_id: usize) {
        let result = if self.recursive {
            pcloud
                .delete_folder_recursive(folder_id)
                .await
                .map(|_| tracing::info!("folder {} deleted", folder_id))
        } else {
            pcloud
                .delete_folder(folder_id)
                .await
                .map(|_| tracing::info!("folder {} deleted", folder_id))
        };
        match result {
            Ok(_) => {
                std::process::exit(exitcode::OK);
            }
            Err(err) => {
                tracing::error!("unable to delete folder: {:?}", err);
                std::process::exit(exitcode::DATAERR);
            }
        }
    }
}
