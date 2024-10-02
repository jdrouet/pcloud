#[cfg(feature = "client-http")]
use crate::client::HttpClient;
#[cfg(feature = "client-http")]
use crate::error::Error;

#[cfg(feature = "client-http")]
#[async_trait::async_trait]
pub trait HttpCommand {
    type Output;

    async fn execute(self, client: &HttpClient) -> Result<Self::Output, Error>;
}
