pub mod create;
pub mod delete;
pub mod list;
pub mod rename;

use crate::entry::Folder;

pub const ROOT: usize = 0;

#[derive(Debug, serde::Deserialize)]
pub(crate) struct FolderResponse {
    metadata: Folder,
}

#[derive(Debug)]
pub enum FolderIdentifier {
    Path(String),
    FolderId(usize),
}

impl Default for FolderIdentifier {
    fn default() -> Self {
        Self::FolderId(0)
    }
}

impl From<&str> for FolderIdentifier {
    fn from(value: &str) -> Self {
        Self::Path(value.to_string())
    }
}

impl From<String> for FolderIdentifier {
    fn from(value: String) -> Self {
        Self::Path(value)
    }
}

impl From<usize> for FolderIdentifier {
    fn from(value: usize) -> Self {
        Self::FolderId(value)
    }
}

impl FolderIdentifier {
    pub fn to_vec(&self) -> Vec<(&str, String)> {
        match self {
            Self::Path(value) => vec![("path", value.clone())],
            Self::FolderId(value) => vec![("folderid", value.to_string())],
        }
    }
}
