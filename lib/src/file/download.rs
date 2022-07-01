//! Resources needed to download a file

use super::FileIdentifier;
use std::io::Write;

/// Command to download a file
///
/// Executing this command with return the size of the downloaded file as a `usize`.
///
/// [More about it on the documentation](https://docs.pcloud.com/methods/file/downloadfile.html)
///
/// # Example using the [`HttpClient`](crate::http::HttpClient)
///
/// To use this, the `client-http` feature should be enabled.
///
/// ```
/// use pcloud::http::HttpClientBuilder;
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
pub struct FileDownloadCommand<W> {
    pub identifier: FileIdentifier,
    pub writer: W,
}

impl<W: Write> FileDownloadCommand<W> {
    pub fn new(identifier: FileIdentifier, writer: W) -> Self {
        Self { identifier, writer }
    }
}

#[cfg(feature = "client-http")]
mod http {
    use super::FileDownloadCommand;
    use crate::error::Error;
    use crate::file::get_link::FileLinkCommand;
    use crate::http::HttpClient;
    use crate::prelude::HttpCommand;
    use std::io::Write;

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
}
