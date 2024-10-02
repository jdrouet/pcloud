pub mod create;
pub mod delete;
pub mod list;
pub mod rename;

pub const ROOT: u64 = 0;

#[cfg(feature = "client-http")]
#[derive(Debug, serde::Deserialize)]
pub(crate) struct FolderResponse {
    pub metadata: crate::entry::Folder,
}

#[derive(Debug)]
pub enum FolderIdentifier {
    Path(String),
    FolderId(u64),
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

impl From<u64> for FolderIdentifier {
    fn from(value: u64) -> Self {
        Self::FolderId(value)
    }
}

#[cfg(feature = "client-http")]
#[derive(serde::Serialize)]
#[serde(untagged)]
pub(crate) enum FolderIdentifierParam {
    Path { path: String },
    FolderId { folderid: u64 },
}

#[cfg(feature = "client-http")]
impl From<FolderIdentifier> for FolderIdentifierParam {
    fn from(value: FolderIdentifier) -> Self {
        match value {
            FolderIdentifier::FolderId(folderid) => Self::FolderId { folderid },
            FolderIdentifier::Path(path) => Self::Path { path },
        }
    }
}
