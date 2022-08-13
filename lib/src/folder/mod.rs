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

#[cfg(feature = "client-binary")]
impl FolderIdentifier {
    pub fn to_named_binary_param(
        &self,
        path: &'static str,
        folder_id: &'static str,
    ) -> (&'static str, BinaryValue) {
        match self {
            Self::Path(value) => (path, BinaryValue::Text(value.clone())),
            Self::FolderId(value) => (folder_id, BinaryValue::Number(*value)),
        }
    }

    pub fn to_binary_params(&self) -> Vec<(&str, BinaryValue)> {
        vec![self.to_binary_param()]
    }

    pub fn to_binary_param(&self) -> (&str, BinaryValue) {
        self.to_named_binary_param("path", "folderid")
    }
}

#[cfg(test)]
mod tests {
    use super::FolderResponse;

    #[test]
    fn decode_example() {
        let example = include_str!("./list-response.json");
        let _decoded: FolderResponse = serde_json::from_str(example).unwrap();
    }
}

#[cfg(all(test, feature = "protected", feature = "client-binary"))]
mod tests_binary {
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
