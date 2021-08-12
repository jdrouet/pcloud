mod file;
mod folder;

use clap::Clap;
use pcloud::http::PCloudApi;

#[derive(Clap)]
enum Command {
    Folder(folder::Command),
    File(file::Command),
}

impl Command {
    async fn execute(&self, pcloud: PCloudApi) {
        match self {
            Self::Folder(sub) => sub.execute(pcloud).await,
            Self::File(sub) => sub.execute(pcloud).await,
        }
    }
}

#[tokio::main]
async fn main() {
    env_logger::init();

    let cmd = Command::parse();
    let pcloud = PCloudApi::from_env();
    cmd.execute(pcloud).await;
}
