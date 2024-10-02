//! Resources needed to rename and move a folder

/// Command to rename a folder
///
/// Executing this command will return a [`Folder`](crate::entry::Folder) on success.
///
/// [More about it on the documentation](https://docs.pcloud.com/methods/folder/renamefolder.html).
///
/// # Example using the [`HttpClient`](crate::client::HttpClient)
///
/// To use this, the `client-http` feature should be enabled.
///
/// ```
/// use pcloud::client::HttpClientBuilder;
/// use pcloud::prelude::HttpCommand;
/// use pcloud::folder::rename::FolderRenameCommand;
///
/// # tokio_test::block_on(async {
/// let client = HttpClientBuilder::from_env().build().unwrap();
/// let cmd = FolderRenameCommand::new(12, "foo".into());
/// match cmd.execute(&client).await {
///   Ok(res) => println!("success"),
///   Err(err) => eprintln!("error: {:?}", err),
/// }
/// # })
/// ```
#[derive(Debug)]
pub struct FolderRenameCommand {
    pub identifier: u64,
    pub name: String,
}

impl FolderRenameCommand {
    pub fn new(identifier: u64, name: String) -> Self {
        Self { identifier, name }
    }
}

/// Command to move a folder
///
/// Executing this command will return a [`Folder`](crate::entry::Folder) on success.
///
/// [More about it on the documentation](https://docs.pcloud.com/methods/folder/renamefolder.html).
///
/// # Example using the [`HttpClient`](crate::client::HttpClient)
///
/// To use this, the `client-http` feature should be enabled.
///
/// ```
/// use pcloud::client::HttpClientBuilder;
/// use pcloud::prelude::HttpCommand;
/// use pcloud::folder::rename::FolderMoveCommand;
///
/// # tokio_test::block_on(async {
/// let client = HttpClientBuilder::from_env().build().unwrap();
/// let cmd = FolderMoveCommand::new(12, 42);
/// match cmd.execute(&client).await {
///   Ok(res) => println!("success"),
///   Err(err) => eprintln!("error: {:?}", err),
/// }
/// # })
/// ```
#[derive(Debug)]
pub struct FolderMoveCommand {
    pub folder: u64,
    pub to_folder: u64,
}

impl FolderMoveCommand {
    pub fn new(folder: u64, to_folder: u64) -> Self {
        Self { folder, to_folder }
    }
}

#[cfg(feature = "client-http")]
mod http {
    use super::{FolderMoveCommand, FolderRenameCommand};
    use crate::client::HttpClient;
    use crate::entry::Folder;
    use crate::error::Error;
    use crate::folder::FolderResponse;
    use crate::prelude::HttpCommand;
    use crate::request::Response;

    #[derive(serde::Serialize)]
    struct FolderRenameParams {
        #[serde(rename = "folderid")]
        folder_id: u64,
        #[serde(rename = "toname")]
        to_name: String,
    }

    impl From<FolderRenameCommand> for FolderRenameParams {
        fn from(value: FolderRenameCommand) -> Self {
            Self {
                folder_id: value.identifier,
                to_name: value.name,
            }
        }
    }

    #[async_trait::async_trait]
    impl HttpCommand for FolderRenameCommand {
        type Output = Folder;

        async fn execute(self, client: &HttpClient) -> Result<Self::Output, Error> {
            let params = FolderRenameParams::from(self);
            let result: Response<FolderResponse> =
                client.get_request("renamefolder", &params).await?;
            result.payload().map(|item| item.metadata)
        }
    }

    #[derive(serde::Serialize)]
    struct FolderMoveParams {
        #[serde(rename = "folderid")]
        folder_id: u64,
        #[serde(rename = "tofolderid")]
        to_folder_id: u64,
    }

    impl From<FolderMoveCommand> for FolderMoveParams {
        fn from(value: FolderMoveCommand) -> Self {
            Self {
                folder_id: value.folder,
                to_folder_id: value.to_folder,
            }
        }
    }

    #[async_trait::async_trait]
    impl HttpCommand for FolderMoveCommand {
        type Output = Folder;

        async fn execute(self, client: &HttpClient) -> Result<Self::Output, Error> {
            let params = FolderMoveParams::from(self);
            let result: Response<FolderResponse> =
                client.get_request("renamefolder", &params).await?;
            result.payload().map(|item| item.metadata)
        }
    }
}
