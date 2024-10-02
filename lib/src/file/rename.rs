//! Resources needed to rename a file

use super::FileIdentifier;
use crate::folder::FolderIdentifier;

/// Command to rename a file
///
/// Executing this command will return a [`File`](crate::entry::File) on success.
///
/// [More about it on the documentation](https://docs.pcloud.com/methods/file/renamefile.html).
///
/// # Example using the [`HttpClient`](crate::client::HttpClient)
///
/// To use this, the `client-http` feature should be enabled.
///
/// ```
/// use pcloud::client::HttpClientBuilder;
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
    use crate::client::HttpClient;
    use crate::entry::File;
    use crate::error::Error;
    use crate::file::{FileIdentifierParam, FileResponse};
    use crate::folder::FolderIdentifier;
    use crate::prelude::HttpCommand;
    use crate::request::Response;

    #[derive(serde::Serialize)]
    #[serde(untagged)]
    enum ToFolderIdentifierParam {
        Path {
            #[serde(rename = "topath")]
            to_path: String,
        },
        FolderId {
            #[serde(rename = "tofolderid")]
            to_folder_id: u64,
        },
    }

    impl From<FolderIdentifier> for ToFolderIdentifierParam {
        fn from(value: FolderIdentifier) -> Self {
            match value {
                FolderIdentifier::Path(to_path) => Self::Path { to_path },
                FolderIdentifier::FolderId(to_folder_id) => Self::FolderId { to_folder_id },
            }
        }
    }

    #[derive(serde::Serialize)]
    struct FileMoveParams {
        #[serde(flatten)]
        from: FileIdentifierParam,
        #[serde(flatten)]
        to: ToFolderIdentifierParam,
    }

    impl From<FileMoveCommand> for FileMoveParams {
        fn from(value: FileMoveCommand) -> Self {
            Self {
                from: FileIdentifierParam::from(value.from),
                to: ToFolderIdentifierParam::from(value.to),
            }
        }
    }

    #[async_trait::async_trait]
    impl HttpCommand for FileMoveCommand {
        type Output = File;

        async fn execute(self, client: &HttpClient) -> Result<Self::Output, Error> {
            let params = FileMoveParams::from(self);
            let result: Response<FileResponse> = client.get_request("renamefile", &params).await?;
            result.payload().map(|item| item.metadata)
        }
    }

    #[derive(serde::Serialize)]
    struct FileRenameParams {
        #[serde(flatten)]
        identifier: FileIdentifierParam,
        #[serde(rename = "toname")]
        to_name: String,
    }

    impl From<FileRenameCommand> for FileRenameParams {
        fn from(value: FileRenameCommand) -> Self {
            Self {
                identifier: FileIdentifierParam::from(value.identifier),
                to_name: value.name,
            }
        }
    }

    #[async_trait::async_trait]
    impl HttpCommand for FileRenameCommand {
        type Output = File;

        async fn execute(self, client: &HttpClient) -> Result<Self::Output, Error> {
            let params = FileRenameParams::from(self);
            let result: Response<FileResponse> = client.get_request("renamefile", &params).await?;
            result.payload().map(|item| item.metadata)
        }
    }
}
