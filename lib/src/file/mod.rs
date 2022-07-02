pub mod checksum;
pub mod copy;
pub mod delete;
pub mod download;
pub mod get_link;
pub mod rename;
pub mod upload;

#[deprecated]
pub use checksum as get_info;

#[cfg(feature = "client-binary")]
use crate::binary::Value as BValue;
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
#[derive(Debug)]
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

impl FileIdentifier {
    #[cfg(feature = "client-http")]
    pub fn to_http_params(&self) -> Vec<(&str, String)> {
        match self {
            Self::Path(value) => vec![("path", value.clone())],
            Self::FileId(value) => vec![("fileid", value.to_string())],
        }
    }

    #[cfg(feature = "client-binary")]
    pub fn to_binary_params(&self) -> Vec<(&str, BValue)> {
        match self {
            Self::Path(value) => vec![("path", BValue::Text(value.clone()))],
            Self::FileId(value) => vec![("fileid", BValue::Number(*value))],
        }
    }
}
