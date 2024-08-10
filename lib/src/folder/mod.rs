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
impl FolderIdentifier {
    pub fn to_named_http_param(
        &self,
        path: &'static str,
        folder_id: &'static str,
    ) -> (&'static str, String) {
        match self {
            Self::Path(value) => (path, value.clone()),
            Self::FolderId(value) => (folder_id, value.to_string()),
        }
    }

    pub fn to_http_params(&self) -> Vec<(&str, String)> {
        vec![self.to_http_param()]
    }

    pub fn to_http_param(&self) -> (&str, String) {
        self.to_named_http_param("path", "folderid")
    }
}
