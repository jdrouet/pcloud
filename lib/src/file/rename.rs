use super::{FileIdentifier, FileResponse};
use crate::binary::{BinaryClient, Value as BinaryValue};
use crate::entry::File;
use crate::error::Error;
use crate::folder::FolderIdentifier;
use crate::http::HttpClient;
use crate::prelude::{BinaryCommand, HttpCommand};
use crate::request::Response;

#[derive(Debug)]
pub struct FileMoveCommand {
    from: FileIdentifier,
    to: FolderIdentifier,
}

impl FileMoveCommand {
    pub fn new(from: FileIdentifier, to: FolderIdentifier) -> Self {
        Self { from, to }
    }

    fn to_http_params(&self) -> Vec<(&str, String)> {
        vec![
            match &self.from {
                FileIdentifier::FileId(id) => ("fileid", id.to_string()),
                FileIdentifier::Path(value) => ("path", value.to_string()),
            },
            match &self.to {
                FolderIdentifier::FolderId(id) => ("tofolderid", id.to_string()),
                FolderIdentifier::Path(value) => ("topath", value.to_string()),
            },
        ]
    }

    fn to_binary_params(&self) -> Vec<(&str, BinaryValue)> {
        vec![
            match &self.from {
                FileIdentifier::FileId(id) => ("fileid", BinaryValue::Number(*id)),
                FileIdentifier::Path(value) => ("path", BinaryValue::Text(value.to_string())),
            },
            match &self.to {
                FolderIdentifier::FolderId(id) => ("tofolderid", BinaryValue::Number(*id)),
                FolderIdentifier::Path(value) => ("topath", BinaryValue::Text(value.to_string())),
            },
        ]
    }
}

#[async_trait::async_trait(?Send)]
impl HttpCommand for FileMoveCommand {
    type Output = File;

    async fn execute(self, client: &HttpClient) -> Result<Self::Output, Error> {
        let result: Response<FileResponse> = client
            .get_request("renamefile", &self.to_http_params())
            .await?;
        result.payload().map(|item| item.metadata)
    }
}

impl BinaryCommand for FileMoveCommand {
    type Output = File;

    fn execute(self, client: &mut BinaryClient) -> Result<Self::Output, Error> {
        let result = client.send_command("renamefile", &self.to_binary_params())?;
        let result: Response<FileResponse> = serde_json::from_value(result)?;
        result.payload().map(|item| item.metadata)
    }
}

#[derive(Debug)]
pub struct FileRenameCommand {
    identifier: FileIdentifier,
    name: String,
}

impl FileRenameCommand {
    pub fn new(identifier: FileIdentifier, name: String) -> Self {
        Self { identifier, name }
    }

    fn to_http_params(&self) -> Vec<(&str, String)> {
        vec![
            match &self.identifier {
                FileIdentifier::FileId(id) => ("fileid", id.to_string()),
                FileIdentifier::Path(value) => ("path", value.to_string()),
            },
            ("toname", self.name.to_string()),
        ]
    }

    fn to_binary_params(&self) -> Vec<(&str, BinaryValue)> {
        vec![
            match &self.identifier {
                FileIdentifier::FileId(id) => ("fileid", BinaryValue::Number(*id)),
                FileIdentifier::Path(value) => ("path", BinaryValue::Text(value.to_string())),
            },
            ("toname", BinaryValue::Text(self.name.to_string())),
        ]
    }
}

#[async_trait::async_trait(?Send)]
impl HttpCommand for FileRenameCommand {
    type Output = File;

    async fn execute(self, client: &HttpClient) -> Result<Self::Output, Error> {
        let result: Response<FileResponse> = client
            .get_request("renamefile", &self.to_http_params())
            .await?;
        result.payload().map(|item| item.metadata)
    }
}

impl BinaryCommand for FileRenameCommand {
    type Output = File;

    fn execute(self, client: &mut BinaryClient) -> Result<Self::Output, Error> {
        let result = client.send_command("renamefile", &self.to_binary_params())?;
        let result: Response<FileResponse> = serde_json::from_value(result)?;
        result.payload().map(|item| item.metadata)
    }
}
