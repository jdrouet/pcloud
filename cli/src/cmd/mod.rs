mod download;
mod list;
mod upload;

#[derive(clap::Parser)]
pub(crate) enum Command {
    Download(download::Command),
    List(list::Command),
    Upload(upload::Command),
}

impl Command {
    pub(crate) async fn execute(self, client: &pcloud::client::HttpClient) -> anyhow::Result<()> {
        match self {
            Self::Download(inner) => inner.execute(client).await,
            Self::List(inner) => inner.execute(client).await,
            Self::Upload(inner) => inner.execute(client).await,
        }
    }
}
