pub mod create;
pub mod delete;
pub mod list;
pub mod rename;

use crate::binary::Value as BinaryValue;
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
    pub fn to_http_params(&self) -> Vec<(&str, String)> {
        match self {
            Self::Path(value) => vec![("path", value.clone())],
            Self::FolderId(value) => vec![("folderid", value.to_string())],
        }
    }

    pub fn to_binary_params(&self) -> Vec<(&str, BinaryValue)> {
        match self {
            Self::Path(value) => vec![("path", BinaryValue::Text(value.clone()))],
            Self::FolderId(value) => vec![("folderid", BinaryValue::Number(*value as u64))],
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::binary::BinaryClient;
    use crate::credentials::Credentials;
    use crate::region::Region;

    #[test]
    fn binary_success() {
        let creds = Credentials::from_env();
        let mut client = BinaryClient::new(creds, Region::eu()).unwrap();
        let folder = client
            .create_folder(&super::create::Params::new(crate::tests::random_name(), 0))
            .unwrap();
        let folder = client
            .rename_folder(&super::rename::Params::new_rename(
                folder.folder_id,
                crate::tests::random_name(),
            ))
            .unwrap();
        client.delete_folder(folder.folder_id).unwrap();
    }
}
