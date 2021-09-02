mod file;
mod folder;

use clap::Clap;
use pcloud::http::HttpClient;

#[derive(Clap)]
enum Command {
    Folder(folder::Command),
    File(file::Command),
}

impl Command {
    async fn execute(&self, pcloud: HttpClient) {
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
    let pcloud = HttpClient::from_env();
    cmd.execute(pcloud).await;
}
