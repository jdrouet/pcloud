use super::FolderResponse;
use crate::binary::{BinaryClient, Value as BinaryValue};
use crate::entry::Folder;
use crate::error::Error;
use crate::http::HttpClient;
use crate::prelude::{BinaryCommand, HttpCommand};
use crate::request::Response;

#[derive(Debug)]
pub struct FolderRenameCommand {
    identifier: u64,
    name: String,
}

impl FolderRenameCommand {
    pub fn new(identifier: u64, name: String) -> Self {
        Self { identifier, name }
    }
}

#[async_trait::async_trait(?Send)]
impl HttpCommand for FolderRenameCommand {
    type Output = Folder;

    async fn execute(self, client: &HttpClient) -> Result<Self::Output, Error> {
        let params = vec![
            ("folderid", self.identifier.to_string()),
            ("toname", self.name),
        ];
        let result: Response<FolderResponse> = client.get_request("renamefolder", &params).await?;
        result.payload().map(|item| item.metadata)
    }
}

impl BinaryCommand for FolderRenameCommand {
    type Output = Folder;

    fn execute(self, client: &mut BinaryClient) -> Result<Self::Output, Error> {
        let params = vec![
            ("folderid", BinaryValue::Number(self.identifier)),
            ("toname", BinaryValue::Text(self.name.clone())),
        ];
        let result = client.send_command("renamefolder", &params)?;
        let result: Response<FolderResponse> = serde_json::from_value(result)?;
        result.payload().map(|item| item.metadata)
    }
}

#[derive(Debug)]
pub struct FolderMoveCommand {
    folder: u64,
    to_folder: u64,
}

impl FolderMoveCommand {
    pub fn new(folder: u64, to_folder: u64) -> Self {
        Self { folder, to_folder }
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
        let result: Response<FolderResponse> = client.get_request("renamefolder", &params).await?;
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
