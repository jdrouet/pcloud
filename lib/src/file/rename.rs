//! Resources needed to rename a file

use super::FileIdentifier;
use crate::folder::FolderIdentifier;

/// Command to rename a file
///
/// Executing this command will return a [`File`](crate::entry::File) on success.
///
/// [More about it on the documentation](https://docs.pcloud.com/methods/file/renamefile.html).
///
/// # Example using the [`HttpClient`](crate::http::HttpClient)
///
/// To use this, the `client-http` feature should be enabled.
///
/// ```
/// use pcloud::http::HttpClientBuilder;
/// use pcloud::prelude::HttpCommand;
/// use pcloud::file::rename::FileRenameCommand;
///
/// # tokio_test::block_on(async {
/// let client = HttpClientBuilder::from_env().build().unwrap();
/// let cmd = FileRenameCommand::new("/foo/bar.txt".into(), "/foo/baz.txt".into());
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
/// use pcloud::file::rename::FileRenameCommand;
///
/// let mut client = BinaryClientBuilder::from_env().build().unwrap();
/// let cmd = FileRenameCommand::new(12.into(), "/foo/baz.txt".into());
/// match cmd.execute(&mut client) {
///   Ok(res) => println!("success"),
///   Err(err) => eprintln!("error: {:?}", err),
/// }
/// ```
#[derive(Debug)]
pub struct FileMoveCommand {
    pub from: FileIdentifier,
    pub to: FolderIdentifier,
}

impl FileMoveCommand {
    pub fn new(from: FileIdentifier, to: FolderIdentifier) -> Self {
        Self { from, to }
    }
}
#[derive(Debug)]
pub struct FileRenameCommand {
    pub identifier: FileIdentifier,
    pub name: String,
}

impl FileRenameCommand {
    pub fn new(identifier: FileIdentifier, name: String) -> Self {
        Self { identifier, name }
    }
}

#[cfg(feature = "client-http")]
mod http {
    use super::{FileMoveCommand, FileRenameCommand};
    use crate::entry::File;
    use crate::error::Error;
    use crate::file::FileResponse;
    use crate::http::HttpClient;
    use crate::prelude::HttpCommand;
    use crate::request::Response;

    impl FileMoveCommand {
        fn to_http_params(&self) -> Vec<(&str, String)> {
            vec![
                self.from.to_http_param(),
                self.to.to_named_http_param("topath", "tofolderid"),
            ]
        }
    }

    #[async_trait::async_trait]
    impl HttpCommand for FileMoveCommand {
        type Output = File;

        async fn execute(self, client: &HttpClient) -> Result<Self::Output, Error> {
            let result: Response<FileResponse> = client
                .get_request("renamefile", &self.to_http_params())
                .await?;
            result.payload().map(|item| item.metadata)
        }
    }

    impl FileRenameCommand {
        fn to_http_params(&self) -> Vec<(&str, String)> {
            vec![
                self.identifier.to_http_param(),
                ("toname", self.name.to_string()),
            ]
        }
    }

    #[async_trait::async_trait]
    impl HttpCommand for FileRenameCommand {
        type Output = File;

        async fn execute(self, client: &HttpClient) -> Result<Self::Output, Error> {
            let result: Response<FileResponse> = client
                .get_request("renamefile", &self.to_http_params())
                .await?;
            result.payload().map(|item| item.metadata)
        }
    }
}
