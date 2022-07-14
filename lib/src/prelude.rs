#[cfg(feature = "client-binary")]
use crate::binary::BinaryClient;
#[cfg(any(feature = "client-binary", feature = "client-http"))]
use crate::error::Error;
#[cfg(feature = "client-http")]
use crate::http::HttpClient;

#[cfg(feature = "client-http")]
#[async_trait::async_trait]
pub trait HttpCommand {
    type Output;

    async fn execute(self, client: &HttpClient) -> Result<Self::Output, Error>;
}

#[cfg(feature = "client-binary")]
pub trait BinaryCommand {
    type Output;

    fn execute(self, client: &mut BinaryClient) -> Result<Self::Output, Error>;
}
