use super::FileIdentifier;
use crate::error::Error;
use crate::file::get_link::FileLinkCommand;
use crate::http::HttpClient;
use crate::prelude::HttpCommand;
use std::io::Write;

#[derive(Debug)]
pub struct FileDownloadCommand<W> {
    identifier: FileIdentifier,
    writer: W,
}

impl<W: Write> FileDownloadCommand<W> {
    pub fn new(identifier: FileIdentifier, writer: W) -> Self {
        Self { identifier, writer }
    }
}

#[async_trait::async_trait(?Send)]
impl<W: Write> HttpCommand for FileDownloadCommand<W> {
    type Output = usize;

    async fn execute(mut self, client: &HttpClient) -> Result<Self::Output, Error> {
        let link = FileLinkCommand::new(self.identifier)
            .execute(client)
            .await?;
        let mut req = client.client.get(&link).send().await?;
        let mut size = 0;
        while let Some(chunk) = req.chunk().await? {
            size += self.writer.write(chunk.as_ref()).map_err(Error::Download)?;
        }
        Ok(size)
    }
}
