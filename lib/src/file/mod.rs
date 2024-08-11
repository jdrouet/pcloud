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
impl FileIdentifier {
    pub fn to_named_http_param(
        &self,
        path: &'static str,
        file_id: &'static str,
    ) -> (&'static str, String) {
        match self {
            Self::Path(value) => (path, value.clone()),
            Self::FileId(value) => (file_id, value.to_string()),
        }
    }

    pub fn to_http_param(&self) -> (&str, String) {
        self.to_named_http_param("path", "fileid")
    }

    pub fn to_http_params(&self) -> Vec<(&str, String)> {
        vec![self.to_http_param()]
    }
}
