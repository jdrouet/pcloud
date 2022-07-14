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
///
/// # Example using the [`BinaryClient`](crate::binary::BinaryClient)
///
/// To use this, the `client-binary` feature should be enabled.
///
/// ```
/// use pcloud::binary::BinaryClientBuilder;
/// use pcloud::prelude::BinaryCommand;
/// use pcloud::file::copy::FileCopyCommand;
///
/// let mut client = BinaryClientBuilder::from_env().build().unwrap();
/// let cmd = FileCopyCommand::new(12, 42);
/// match cmd.execute(&mut client) {
///   Ok(res) => println!("success"),
///   Err(err) => eprintln!("error: {:?}", err),
/// }
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

    impl FileCopyCommand {
        fn to_http_params(&self) -> Vec<(&str, String)> {
            vec![
                ("fileid", self.file_id.to_string()),
                ("tofolderid", self.to_folder_id.to_string()),
            ]
        }
    }

    #[async_trait::async_trait]
    impl HttpCommand for FileCopyCommand {
        type Output = File;

        async fn execute(self, client: &HttpClient) -> Result<Self::Output, Error> {
            let result: Response<FileResponse> = client
                .get_request("copyfile", &self.to_http_params())
                .await?;
            result.payload().map(|item| item.metadata)
        }
    }
}

#[cfg(feature = "client-binary")]
mod binary {
    use super::FileCopyCommand;
    use crate::binary::{BinaryClient, Value as BinaryValue};
    use crate::entry::File;
    use crate::error::Error;
    use crate::file::FileResponse;
    use crate::prelude::BinaryCommand;
    use crate::request::Response;

    impl FileCopyCommand {
        fn to_binary_params(&self) -> Vec<(&str, BinaryValue)> {
            vec![
                ("fileid", BinaryValue::Number(self.file_id)),
                ("tofolderid", BinaryValue::Number(self.to_folder_id)),
            ]
        }
    }

    impl BinaryCommand for FileCopyCommand {
        type Output = File;

        fn execute(self, client: &mut BinaryClient) -> Result<Self::Output, Error> {
            let result = client.send_command("copyfile", &self.to_binary_params())?;
            let result: Response<FileResponse> = serde_json::from_value(result)?;
            result.payload().map(|item| item.metadata)
        }
    }
}
