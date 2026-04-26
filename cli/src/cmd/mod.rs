mod download;
mod list;
mod oauth;

#[derive(clap::Parser)]
pub(crate) enum Command {
    Download(download::Command),
    List(list::Command),
    /// Run the OAuth2 authorization flow against pCloud, exchanging the
    /// authorization code for an access token via a local HTTP listener.
    Oauth(oauth::Command),
}

impl Command {
    pub(crate) async fn execute(self, client: &pcloud::Client) -> anyhow::Result<()> {
        match self {
            Self::Download(inner) => inner.execute(client).await,
            Self::List(inner) => inner.execute(client).await,
            Self::Oauth(inner) => inner.execute(client).await,
        }
    }
}
