use super::FolderResponse;
use crate::binary::{BinaryClient, Value as BinaryValue};
use crate::entry::Folder;
use crate::error::Error;
use crate::http::HttpClient;
use crate::prelude::Command;
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
impl Command for FolderRenameCommand {
    type Output = Folder;
    type Error = Error;

    async fn execute(self, client: &HttpClient) -> Result<Self::Output, Self::Error> {
        let params = vec![
            ("folderid", self.identifier.to_string()),
            ("toname", self.name),
        ];
        let result: Response<FolderResponse> = client.get_request("renamefolder", &params).await?;
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
impl Command for FolderMoveCommand {
    type Output = Folder;
    type Error = Error;

    async fn execute(self, client: &HttpClient) -> Result<Self::Output, Self::Error> {
        let params = vec![
            ("folderid", self.folder.to_string()),
            ("tofolderid", self.to_folder.to_string()),
        ];
        let result: Response<FolderResponse> = client.get_request("renamefolder", &params).await?;
        result.payload().map(|item| item.metadata)
    }
}

#[derive(Debug)]
pub enum Params {
    Rename { folder_id: u64, name: String },
    Move { folder_id: u64, to_folder_id: u64 },
}

impl Params {
    pub fn new_rename<S: Into<String>>(folder_id: u64, name: S) -> Self {
        Self::Rename {
            folder_id,
            name: name.into(),
        }
    }

    pub fn new_move(folder_id: u64, to_folder_id: u64) -> Self {
        Self::Move {
            folder_id,
            to_folder_id,
        }
    }

    pub fn to_binary_params(&self) -> Vec<(&str, BinaryValue)> {
        match self {
            Self::Rename { folder_id, name } => vec![
                ("folderid", BinaryValue::Number(*folder_id)),
                ("toname", BinaryValue::Text(name.to_string())),
            ],
            Self::Move {
                folder_id,
                to_folder_id,
            } => vec![
                ("folderid", BinaryValue::Number(*folder_id)),
                ("tofolderid", BinaryValue::Number(*to_folder_id)),
            ],
        }
    }
}

impl BinaryClient {
    #[tracing::instrument(skip(self))]
    pub fn rename_folder(&mut self, params: &Params) -> Result<Folder, Error> {
        let result = self.send_command("renamefolder", &params.to_binary_params())?;
        let result: Response<FolderResponse> = serde_json::from_value(result)?;
        result.payload().map(|item| item.metadata)
    }
}
