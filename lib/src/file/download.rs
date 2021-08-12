use crate::error::Error;
use crate::http::PCloudApi;
use std::io::Write;

impl PCloudApi {
    /// Download a file
    ///
    /// # Arguments
    ///
    /// * `file_id` - ID of the file to download.
    /// * `write` - Where to write the file.
    ///
    pub async fn download_file<W: Write>(
        &self,
        file_id: usize,
        mut write: W,
    ) -> Result<usize, Error> {
        let link = self.get_link_file(file_id).await?;
        let mut req = self.client.get(&link).send().await?;
        let mut size = 0;
        while let Some(chunk) = req.chunk().await? {
            size += write.write(chunk.as_ref()).map_err(Error::Download)?;
        }
        Ok(size)
    }
}
