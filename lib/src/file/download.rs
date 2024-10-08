//! Resources needed to download a file

use super::FileIdentifier;
use std::io::Write;

/// Command to download a file
///
/// Executing this command with return the size of the downloaded file as a `usize`.
///
/// [More about it on the documentation](https://docs.pcloud.com/methods/file/downloadfile.html)
///
/// # Example using the [`HttpClient`](crate::client::HttpClient)
///
/// To use this, the `client-http` feature should be enabled.
///
/// ```
/// use pcloud::client::HttpClientBuilder;
/// use pcloud::prelude::HttpCommand;
/// use pcloud::file::download::FileDownloadCommand;
/// use std::fs::File;
///
/// # tokio_test::block_on(async {
/// let file = File::create("./output.txt").unwrap();
/// let client = HttpClientBuilder::from_env().build().unwrap();
/// let cmd = FileDownloadCommand::new("/foo/bar.txt".into(), file);
/// match cmd.execute(&client).await {
///   Ok(res) => println!("success"),
///   Err(err) => eprintln!("error: {:?}", err),
/// }
/// # })
/// ```
#[derive(Debug)]
pub struct FileDownloadCommand<'a, W> {
    pub identifier: FileIdentifier<'a>,
    pub writer: W,
}

impl<'a, W: Write> FileDownloadCommand<'a, W> {
    pub fn new(identifier: FileIdentifier<'a>, writer: W) -> Self {
        Self { identifier, writer }
    }
}

#[cfg(feature = "client-http")]
mod http {
    use super::FileDownloadCommand;
    use crate::client::HttpClient;
    use crate::error::Error;
    use crate::prelude::HttpCommand;
    use crate::streaming::get_file_link::GetFileLinkCommand;
    use std::io::Write;

    #[async_trait::async_trait]
    impl<'a, W: Write + Send> HttpCommand for FileDownloadCommand<'a, W> {
        type Output = usize;

        async fn execute(mut self, client: &HttpClient) -> Result<Self::Output, Error> {
            let links = GetFileLinkCommand::new(self.identifier)
                .execute(client)
                .await?;
            let link = links.first_link().ok_or_else(|| {
                Error::Download(std::io::Error::new(
                    std::io::ErrorKind::NotFound,
                    "unable to find a link for requested file",
                ))
            })?;
            let link = link.to_string();
            let mut req = client.client.get(&link).send().await?;
            let mut size = 0;
            while let Some(chunk) = req.chunk().await? {
                size += self.writer.write(chunk.as_ref()).map_err(Error::Download)?;
            }
            Ok(size)
        }
    }
}
