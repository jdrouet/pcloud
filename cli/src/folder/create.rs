use clap::Parser;
use pcloud::folder::create::Params;
use pcloud::http::HttpClient;

#[derive(Parser)]
pub struct Command {
    name: String,
}

impl Command {
    #[tracing::instrument(skip_all)]
    pub async fn execute(&self, pcloud: HttpClient, folder_id: usize) {
        let params = Params::new(&self.name, folder_id);
        match pcloud.create_folder(&params).await {
            Ok(res) => {
                tracing::info!("folder created {}", res.folder_id);
                std::process::exit(exitcode::OK);
            }
            Err(err) => {
                tracing::error!("unable to create folder: {:?}", err);
                std::process::exit(exitcode::CANTCREAT);
            }
        }
    }
}
