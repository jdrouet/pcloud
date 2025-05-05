use std::{borrow::Cow, cmp::Ordering};

use serde::ser::SerializeStruct;

use crate::entry::EntryBase;

pub mod checksum;
pub mod delete;
pub mod movefile; // Can't name it "move" as it's a reserved keyword
pub mod rename;
pub mod upload;

/// Response returned when moving or copying a file.
///
/// This struct wraps file metadata after an operation that results
/// in a file being created or relocated (e.g., move, copy).
#[derive(Debug, serde::Deserialize)]
pub struct FileResponse {
    /// Metadata of the resulting file after the operation.
    pub metadata: File,
}

/// A file identifier used in API calls.
///
/// Files on pCloud can be referenced either by their **file ID** or by their **path**.
/// File IDs are preferred for reliability and performance.
///
/// This enum allows you to pass either type of identifier interchangeably.
#[derive(Clone, Debug)]
pub enum FileIdentifier<'a> {
    /// A file path (e.g., `"folder/subfolder/file.txt"`).
    Path(Cow<'a, str>),

    /// A file ID assigned by pCloud.
    FileId(u64),
}

impl<'a> FileIdentifier<'a> {
    /// Creates a [`FileIdentifier`] from a path.
    #[inline]
    pub fn path<P: Into<Cow<'a, str>>>(path: P) -> Self {
        Self::Path(path.into())
    }

    /// Creates a [`FileIdentifier`] from a file ID.
    #[inline]
    pub fn file_id(fileid: u64) -> Self {
        Self::FileId(fileid)
    }
}

impl Default for FileIdentifier<'_> {
    fn default() -> Self {
        Self::FileId(0)
    }
}

impl<'a> From<Cow<'a, str>> for FileIdentifier<'a> {
    fn from(value: Cow<'a, str>) -> Self {
        Self::Path(value)
    }
}

impl<'a> From<&'a str> for FileIdentifier<'a> {
    fn from(value: &'a str) -> Self {
        Self::Path(Cow::Borrowed(value))
    }
}

impl<'a> From<&'a String> for FileIdentifier<'a> {
    fn from(value: &'a String) -> Self {
        Self::Path(Cow::Borrowed(value))
    }
}

impl From<String> for FileIdentifier<'_> {
    fn from(value: String) -> Self {
        Self::Path(Cow::Owned(value))
    }
}

impl From<u64> for FileIdentifier<'_> {
    fn from(value: u64) -> Self {
        Self::FileId(value)
    }
}

impl serde::Serialize for FileIdentifier<'_> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let mut builder = serializer.serialize_struct(stringify!(FileIdentifier), 1)?;
        match self {
            Self::FileId(file_id) => {
                builder.serialize_field("fileid", file_id)?;
            }
            Self::Path(path) => {
                builder.serialize_field("path", path)?;
            }
        }
        builder.end()
    }
}

/// A structure representing a file stored on pCloud.
///
/// Includes metadata such as the file's unique ID, size, content type, and other attributes.
///
/// This struct implements comparison traits so files can be sorted or compared by name.
#[derive(Clone, Debug, serde::Deserialize, serde::Serialize)]
pub struct File {
    /// Base metadata common to all entries (files and folders).
    #[serde(flatten)]
    pub base: EntryBase,

    /// The unique file ID assigned by pCloud.
    #[serde(rename = "fileid")]
    pub file_id: u64,

    /// The size of the file in bytes.
    pub size: Option<usize>,

    /// A hash of the file content (may be used for caching or deduplication).
    pub hash: Option<usize>,

    /// The MIME type of the file (e.g., `"image/jpeg"`, `"application/pdf"`).
    #[serde(rename = "contenttype")]
    pub content_type: Option<String>,
}

impl Eq for File {}

impl PartialEq for File {
    fn eq(&self, other: &Self) -> bool {
        self.base.id.eq(&other.base.id)
    }
}

impl Ord for File {
    fn cmp(&self, other: &Self) -> Ordering {
        self.base.name.cmp(&other.base.name)
    }
}

impl PartialOrd for File {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}
