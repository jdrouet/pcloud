pub mod checksum;
pub mod copy;
pub mod delete;
pub mod download;
pub mod rename;
pub mod upload;

use crate::entry::File;

/// Structure returned when moving or copying a file
#[derive(Debug, serde::Deserialize)]
pub struct FileResponse {
    pub metadata: File,
}

/// Representation of a file identifier.
///
/// In most commands, a file can be identifier by it's path,
/// although it's not recommended, or by it id
#[derive(Clone, Debug)]
pub enum FileIdentifier {
    Path(String),
    FileId(u64),
}

impl Default for FileIdentifier {
    fn default() -> Self {
        Self::FileId(0)
    }
}

impl From<&str> for FileIdentifier {
    fn from(value: &str) -> Self {
        Self::Path(value.to_string())
    }
}

impl From<String> for FileIdentifier {
    fn from(value: String) -> Self {
        Self::Path(value)
    }
}

impl From<u64> for FileIdentifier {
    fn from(value: u64) -> Self {
        Self::FileId(value)
    }
}

#[cfg(feature = "client-http")]
#[derive(serde::Serialize)]
#[serde(untagged)]
pub(crate) enum FileIdentifierParam {
    Path { path: String },
    FileId { fileid: u64 },
}

#[cfg(feature = "client-http")]
impl From<FileIdentifier> for FileIdentifierParam {
    fn from(value: FileIdentifier) -> Self {
        match value {
            FileIdentifier::FileId(fileid) => Self::FileId { fileid },
            FileIdentifier::Path(path) => Self::Path { path },
        }
    }
}
