mod list;

#[derive(clap::Parser)]
pub(crate) enum Command {
    List(list::Command),
}

impl Command {
    pub(crate) async fn execute(self, client: &pcloud::client::HttpClient) -> anyhow::Result<()> {
        match self {
            Self::List(inner) => inner.execute(client).await,
        }
    }
}
