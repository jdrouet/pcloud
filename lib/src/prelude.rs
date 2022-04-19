use crate::http::HttpClient;

#[async_trait::async_trait(?Send)]
pub trait Command {
    type Output;
    type Error;

    async fn execute(self, client: &HttpClient) -> Result<Self::Output, Self::Error>;
}
