use super::{FileIdentifier, FileResponse};
use crate::entry::File;
use crate::error::Error;
use crate::folder::FolderIdentifier;
use crate::http::PCloudApi;
use crate::request::Response;

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

    pub fn to_vec(&self) -> Vec<(&str, String)> {
        let mut res = vec![];
        match self {
            Self::Move { from, to } => {
                match from {
                    FileIdentifier::FileId(id) => {
                        res.push(("fileid", id.to_string()));
                    }
                    FileIdentifier::Path(value) => {
                        res.push(("path", value.to_string()));
                    }
                };
                match to {
                    FolderIdentifier::FolderId(id) => {
                        res.push(("tofolderid", id.to_string()));
                    }
                    FolderIdentifier::Path(value) => {
                        res.push(("topath", value.to_string()));
                    }
                };
            }
            Self::Rename { from, to_name } => {
                match from {
                    FileIdentifier::FileId(id) => {
                        res.push(("fileid", id.to_string()));
                    }
                    FileIdentifier::Path(value) => {
                        res.push(("path", value.to_string()));
                    }
                };
                res.push(("toname", to_name.to_string()));
            }
        }
        res
    }
}

impl PCloudApi {
    pub async fn rename_file(&self, params: &Params) -> Result<File, Error> {
        let result: Response<FileResponse> =
            self.get_request("renamefile", &params.to_vec()).await?;
        result.payload().map(|item| item.metadata)
    }
}
