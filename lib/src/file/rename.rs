use super::{FileIdentifier, FileResponse};
use crate::binary::{BinaryClient, Value as BinaryValue};
use crate::entry::File;
use crate::error::Error;
use crate::folder::FolderIdentifier;
use crate::http::HttpClient;
use crate::prelude::HttpCommand;
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
        let mut res = vec![];
        res.push(match &self.from {
            FileIdentifier::FileId(id) => ("fileid", id.to_string()),
            FileIdentifier::Path(value) => ("path", value.to_string()),
        });
        res.push(match &self.to {
            FolderIdentifier::FolderId(id) => ("tofolderid", id.to_string()),
            FolderIdentifier::Path(value) => ("topath", value.to_string()),
        });
        res
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
        let mut res = vec![];
        res.push(match &self.identifier {
            FileIdentifier::FileId(id) => ("fileid", id.to_string()),
            FileIdentifier::Path(value) => ("path", value.to_string()),
        });
        res.push(("toname", self.name.to_string()));
        res
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

#[derive(Debug)]
pub enum Params {
    Move {
        from: FileIdentifier,
        to: FolderIdentifier,
    },
    Rename {
        from: FileIdentifier,
        to_name: String,
    },
}

impl Params {
    pub fn new_move<File: Into<FileIdentifier>, Folder: Into<FolderIdentifier>>(
        from: File,
        to: Folder,
    ) -> Self {
        Self::Move {
            from: from.into(),
            to: to.into(),
        }
    }

    pub fn new_rename<I: Into<FileIdentifier>, S: Into<String>>(from: I, to_name: S) -> Self {
        Self::Rename {
            from: from.into(),
            to_name: to_name.into(),
        }
    }

    pub fn to_binary_params(&self) -> Vec<(&str, BinaryValue)> {
        let mut res = vec![];
        match self {
            Self::Move { from, to } => {
                match from {
                    FileIdentifier::FileId(id) => {
                        res.push(("fileid", BinaryValue::Number(*id)));
                    }
                    FileIdentifier::Path(value) => {
                        res.push(("path", BinaryValue::Text(value.to_string())));
                    }
                };
                match to {
                    FolderIdentifier::FolderId(id) => {
                        res.push(("tofolderid", BinaryValue::Number(*id)));
                    }
                    FolderIdentifier::Path(value) => {
                        res.push(("topath", BinaryValue::Text(value.to_string())));
                    }
                };
            }
            Self::Rename { from, to_name } => {
                match from {
                    FileIdentifier::FileId(id) => {
                        res.push(("fileid", BinaryValue::Number(*id)));
                    }
                    FileIdentifier::Path(value) => {
                        res.push(("path", BinaryValue::Text(value.to_string())));
                    }
                };
                res.push(("toname", BinaryValue::Text(to_name.to_string())));
            }
        }
        res
    }
}

impl BinaryClient {
    #[tracing::instrument(skip(self))]
    pub fn rename_file(&mut self, params: &Params) -> Result<File, Error> {
        let result = self.send_command("renamefile", &params.to_binary_params())?;
        let result: Response<FileResponse> = serde_json::from_value(result)?;
        result.payload().map(|item| item.metadata)
    }
}
