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
    use crate::entry::Folder;
    use crate::error::Error;
    use crate::folder::FolderResponse;
    use crate::http::HttpClient;
    use crate::prelude::HttpCommand;
    use crate::request::Response;

    #[async_trait::async_trait(?Send)]
    impl HttpCommand for FolderRenameCommand {
        type Output = Folder;

        async fn execute(self, client: &HttpClient) -> Result<Self::Output, Error> {
            let params = vec![
                ("folderid", self.identifier.to_string()),
                ("toname", self.name),
            ];
            let result: Response<FolderResponse> =
                client.get_request("renamefolder", &params).await?;
            result.payload().map(|item| item.metadata)
        }
    }

    #[async_trait::async_trait(?Send)]
    impl HttpCommand for FolderMoveCommand {
        type Output = Folder;

        async fn execute(self, client: &HttpClient) -> Result<Self::Output, Error> {
            let params = vec![
                ("folderid", self.folder.to_string()),
                ("tofolderid", self.to_folder.to_string()),
            ];
            let result: Response<FolderResponse> =
                client.get_request("renamefolder", &params).await?;
            result.payload().map(|item| item.metadata)
        }
    }
}

#[cfg(feature = "client-binary")]
mod binary {
    use super::{FolderMoveCommand, FolderRenameCommand};
    use crate::binary::{BinaryClient, Value as BinaryValue};
    use crate::entry::Folder;
    use crate::error::Error;
    use crate::folder::FolderResponse;
    use crate::prelude::BinaryCommand;
    use crate::request::Response;

    impl BinaryCommand for FolderRenameCommand {
        type Output = Folder;

        fn execute(self, client: &mut BinaryClient) -> Result<Self::Output, Error> {
            let params = vec![
                ("folderid", BinaryValue::Number(self.identifier)),
                ("toname", BinaryValue::Text(self.name)),
            ];
            let result = client.send_command("renamefolder", &params)?;
            let result: Response<FolderResponse> = serde_json::from_value(result)?;
            result.payload().map(|item| item.metadata)
        }
    }

    impl BinaryCommand for FolderMoveCommand {
        type Output = Folder;

        fn execute(self, client: &mut BinaryClient) -> Result<Self::Output, Error> {
            let params = vec![
                ("folderid", BinaryValue::Number(self.folder)),
                ("tofolderid", BinaryValue::Number(self.to_folder)),
            ];
            let result = client.send_command("renamefolder", &params)?;
            let result: Response<FolderResponse> = serde_json::from_value(result)?;
            result.payload().map(|item| item.metadata)
        }
    }
}
