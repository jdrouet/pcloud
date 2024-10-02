pub mod checksum;
pub mod copy;
pub mod delete;
pub mod download;
pub mod rename;
pub mod upload;

use std::borrow::Cow;

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
pub enum FileIdentifier<'a> {
    Path(Cow<'a, str>),
    FileId(u64),
}

impl<'a> FileIdentifier<'a> {
    #[inline]
    pub fn path<P: Into<Cow<'a, str>>>(path: P) -> Self {
        Self::Path(path.into())
    }

    #[inline]
    pub fn file_id(fileid: u64) -> Self {
        Self::FileId(fileid)
    }
}

impl<'a> Default for FileIdentifier<'a> {
    fn default() -> Self {
        Self::FileId(0)
    }
}

impl<'a> From<&'a str> for FileIdentifier<'a> {
    fn from(value: &'a str) -> Self {
        Self::Path(Cow::Borrowed(value))
    }
}

impl<'a> From<String> for FileIdentifier<'a> {
    fn from(value: String) -> Self {
        Self::Path(Cow::Owned(value))
    }
}

impl<'a> From<u64> for FileIdentifier<'a> {
    fn from(value: u64) -> Self {
        Self::FileId(value)
    }
}

#[cfg(feature = "client-http")]
#[derive(serde::Serialize)]
#[serde(untagged)]
pub(crate) enum FileIdentifierParam<'a> {
    Path { path: Cow<'a, str> },
    FileId { fileid: u64 },
}

#[cfg(feature = "client-http")]
impl<'a> From<FileIdentifier<'a>> for FileIdentifierParam<'a> {
    fn from(value: FileIdentifier<'a>) -> Self {
        match value {
            FileIdentifier::FileId(fileid) => Self::FileId { fileid },
            FileIdentifier::Path(path) => Self::Path { path },
        }
    }
}
