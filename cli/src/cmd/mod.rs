mod download;
mod list;

#[derive(clap::Parser)]
pub(crate) enum Command {
    Download(download::Command),
    List(list::Command),
}

impl Command {
    pub(crate) async fn execute(self, client: &pcloud::Client) -> anyhow::Result<()> {
        match self {
            Self::Download(inner) => inner.execute(client).await,
            Self::List(inner) => inner.execute(client).await,
        }
    }
}
