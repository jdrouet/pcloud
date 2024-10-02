//! Resources needed to copy a file

/// Command to copy a file to a defined folder
///
/// Executing this command will return a [`File`](crate::entry::File) on success.
///
/// [More about it on the documentation](https://docs.pcloud.com/methods/file/copyfile.html).
///
/// # Example using the [`HttpClient`](crate::http::HttpClient)
///
/// To use this, the `client-http` feature should be enabled.
///
/// ```
/// use pcloud::http::HttpClientBuilder;
/// use pcloud::prelude::HttpCommand;
/// use pcloud::file::copy::FileCopyCommand;
///
/// # tokio_test::block_on(async {
/// let client = HttpClientBuilder::from_env().build().unwrap();
/// let cmd = FileCopyCommand::new(12, 42);
/// match cmd.execute(&client).await {
///   Ok(res) => println!("success"),
///   Err(err) => eprintln!("error: {:?}", err),
/// }
/// # })
/// ```
#[derive(Debug)]
pub struct FileCopyCommand {
    pub file_id: u64,
    pub to_folder_id: u64,
}

impl FileCopyCommand {
    pub fn new(file_id: u64, to_folder_id: u64) -> Self {
        Self {
            file_id,
            to_folder_id,
        }
    }
}

#[cfg(feature = "client-http")]
mod http {
    use super::FileCopyCommand;
    use crate::entry::File;
    use crate::error::Error;
    use crate::file::FileResponse;
    use crate::http::HttpClient;
    use crate::prelude::HttpCommand;
    use crate::request::Response;

    #[derive(serde::Serialize)]
    struct FileCopyParams {
        #[serde(rename = "fileid")]
        file_id: u64,
        #[serde(rename = "tofolderid")]
        to_folder_id: u64,
    }

    impl From<FileCopyCommand> for FileCopyParams {
        fn from(value: FileCopyCommand) -> Self {
            Self {
                file_id: value.file_id,
                to_folder_id: value.to_folder_id,
            }
        }
    }

    #[async_trait::async_trait]
    impl HttpCommand for FileCopyCommand {
        type Output = File;

        async fn execute(self, client: &HttpClient) -> Result<Self::Output, Error> {
            let params = FileCopyParams::from(self);
            let result: Response<FileResponse> = client.get_request("copyfile", &params).await?;
            result.payload().map(|item| item.metadata)
        }
    }
}
