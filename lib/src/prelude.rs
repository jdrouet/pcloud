use crate::binary::BinaryClient;
use crate::error::Error;
use crate::http::HttpClient;

#[async_trait::async_trait(?Send)]
pub trait HttpCommand {
    type Output;

    async fn execute(self, client: &HttpClient) -> Result<Self::Output, Error>;
}

pub trait BinaryCommand {
    type Output;

    fn execute(self, client: &mut BinaryClient) -> Result<Self::Output, Error>;
}
