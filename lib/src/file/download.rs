use super::FileIdentifier;
use crate::error::Error;
use crate::http::HttpClient;
use std::io::Write;

impl HttpClient {
    /// Download a file
    ///
    /// # Arguments
    ///
    /// * `identifier` - ID or path to the file to download.
    /// * `write` - Where to write the file.
    ///
    #[tracing::instrument(skip(self, write))]
    pub async fn download_file<I: Into<FileIdentifier> + std::fmt::Debug, W: Write>(
        &self,
        identifier: I,
        mut write: W,
    ) -> Result<usize, Error> {
        let link = self.get_link_file(identifier).await?;
        let mut req = self.client.get(&link).send().await?;
        let mut size = 0;
        while let Some(chunk) = req.chunk().await? {
            size += write.write(chunk.as_ref()).map_err(Error::Download)?;
        }
        Ok(size)
    }
}
