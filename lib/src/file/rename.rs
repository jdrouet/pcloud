//! Resources needed to rename a file

use std::borrow::Cow;

use super::FileIdentifier;
use crate::folder::FolderIdentifier;

/// Command to move a file
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
/// use pcloud::file::FileIdentifier;
/// use pcloud::file::rename::FileMoveCommand;
/// use pcloud::folder::FolderIdentifier;
///
/// # tokio_test::block_on(async {
/// let client = HttpClientBuilder::from_env().build().unwrap();
/// let cmd = FileMoveCommand::new(FileIdentifier::path("/foo/bar"), FolderIdentifier::path("/foz"));
/// match cmd.execute(&client).await {
///   Ok(res) => println!("success"),
///   Err(err) => eprintln!("error: {:?}", err),
/// }
/// # })
/// ```
#[derive(Debug)]
pub struct FileMoveCommand<'a> {
    pub from: FileIdentifier<'a>,
    pub to: FolderIdentifier<'a>,
}

impl<'a> FileMoveCommand<'a> {
    pub fn new(from: FileIdentifier<'a>, to: FolderIdentifier<'a>) -> Self {
        Self { from, to }
    }
}

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
/// use pcloud::file::FileIdentifier;
/// use pcloud::file::rename::FileRenameCommand;
///
/// # tokio_test::block_on(async {
/// let client = HttpClientBuilder::from_env().build().unwrap();
/// let cmd = FileRenameCommand::new(FileIdentifier::path("/foo/bar"), "/foo/baz.txt");
/// match cmd.execute(&client).await {
///   Ok(res) => println!("success"),
///   Err(err) => eprintln!("error: {:?}", err),
/// }
/// # })
/// ```
#[derive(Debug)]
pub struct FileRenameCommand<'a> {
    pub identifier: FileIdentifier<'a>,
    pub name: Cow<'a, str>,
}

impl<'a> FileRenameCommand<'a> {
    pub fn new<N: Into<Cow<'a, str>>>(identifier: FileIdentifier<'a>, name: N) -> Self {
        Self {
            identifier,
            name: name.into(),
        }
    }
}

#[cfg(feature = "client-http")]
mod http {
    use std::borrow::Cow;

    use super::{FileMoveCommand, FileRenameCommand};
    use crate::client::HttpClient;
    use crate::entry::File;
    use crate::error::Error;
    use crate::file::{FileIdentifierParam, FileResponse};
    use crate::folder::FolderIdentifier;
    use crate::prelude::HttpCommand;

    #[derive(serde::Serialize)]
    #[serde(untagged)]
    enum ToFolderIdentifierParam<'a> {
        Path {
            #[serde(rename = "topath")]
            to_path: Cow<'a, str>,
        },
        FolderId {
            #[serde(rename = "tofolderid")]
            to_folder_id: u64,
        },
    }

    impl<'a> From<FolderIdentifier<'a>> for ToFolderIdentifierParam<'a> {
        fn from(value: FolderIdentifier<'a>) -> Self {
            match value {
                FolderIdentifier::Path(to_path) => Self::Path { to_path },
                FolderIdentifier::FolderId(to_folder_id) => Self::FolderId { to_folder_id },
            }
        }
    }

    #[derive(serde::Serialize)]
    struct FileMoveParams<'a> {
        #[serde(flatten)]
        from: FileIdentifierParam<'a>,
        #[serde(flatten)]
        to: ToFolderIdentifierParam<'a>,
    }

    impl<'a> From<FileMoveCommand<'a>> for FileMoveParams<'a> {
        fn from(value: FileMoveCommand<'a>) -> Self {
            Self {
                from: FileIdentifierParam::from(value.from),
                to: ToFolderIdentifierParam::from(value.to),
            }
        }
    }

    #[async_trait::async_trait]
    impl<'a> HttpCommand for FileMoveCommand<'a> {
        type Output = File;

        async fn execute(self, client: &HttpClient) -> Result<Self::Output, Error> {
            let params = FileMoveParams::from(self);
            client
                .get_request::<FileResponse, _>("renamefile", &params)
                .await
                .map(|item| item.metadata)
        }
    }

    #[derive(serde::Serialize)]
    struct FileRenameParams<'a> {
        #[serde(flatten)]
        identifier: FileIdentifierParam<'a>,
        #[serde(rename = "toname")]
        to_name: Cow<'a, str>,
    }

    impl<'a> From<FileRenameCommand<'a>> for FileRenameParams<'a> {
        fn from(value: FileRenameCommand<'a>) -> Self {
            Self {
                identifier: FileIdentifierParam::from(value.identifier),
                to_name: value.name,
            }
        }
    }

    #[async_trait::async_trait]
    impl<'a> HttpCommand for FileRenameCommand<'a> {
        type Output = File;

        async fn execute(self, client: &HttpClient) -> Result<Self::Output, Error> {
            client
                .get_request::<FileResponse, _>("renamefile", FileRenameParams::from(self))
                .await
                .map(|item| item.metadata)
        }
    }
}
