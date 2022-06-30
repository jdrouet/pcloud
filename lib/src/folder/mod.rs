pub mod create;
pub mod delete;
pub mod list;
pub mod rename;

#[cfg(feature = "client-binary")]
use crate::binary::Value as BinaryValue;

pub const ROOT: u64 = 0;

#[cfg(any(feature = "client-binary", feature = "client-http"))]
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

impl FolderIdentifier {
    #[cfg(feature = "client-http")]
    pub fn to_http_params(&self) -> Vec<(&str, String)> {
        match self {
            Self::Path(value) => vec![("path", value.clone())],
            Self::FolderId(value) => vec![("folderid", value.to_string())],
        }
    }

    #[cfg(feature = "client-binary")]
    pub fn to_binary_params(&self) -> Vec<(&str, BinaryValue)> {
        match self {
            Self::Path(value) => vec![("path", BinaryValue::Text(value.clone()))],
            Self::FolderId(value) => vec![("folderid", BinaryValue::Number(*value))],
        }
    }
}

#[cfg(all(test, feature = "protected", feature = "client-binary"))]
mod tests {
    use crate::binary::BinaryClientBuilder;
    use crate::prelude::BinaryCommand;

    #[test]
    fn binary_success() {
        let mut client = BinaryClientBuilder::from_env().build().unwrap();
        let folder = super::create::FolderCreateCommand::new(crate::tests::random_name(), 0)
            .execute(&mut client)
            .unwrap();
        let folder =
            super::rename::FolderRenameCommand::new(folder.folder_id, crate::tests::random_name())
                .execute(&mut client)
                .unwrap();
        super::delete::FolderDeleteCommand::new(folder.folder_id.into())
            .execute(&mut client)
            .unwrap();
    }
}
