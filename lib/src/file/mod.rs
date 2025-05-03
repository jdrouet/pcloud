use std::{borrow::Cow, cmp::Ordering};

use serde::ser::SerializeStruct;

use crate::entry::EntryBase;

pub mod checksum;
pub mod delete;
pub mod movefile; // can't name it move
pub mod rename;
pub mod upload;

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

impl Default for FileIdentifier<'_> {
    fn default() -> Self {
        Self::FileId(0)
    }
}

impl<'a> From<&'a str> for FileIdentifier<'a> {
    fn from(value: &'a str) -> Self {
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

/// A structure representing a file on PCloud
#[derive(Clone, Debug, serde::Deserialize, serde::Serialize)]
pub struct File {
    #[serde(flatten)]
    pub base: EntryBase,
    #[serde(rename = "fileid")]
    pub file_id: u64,
    pub size: Option<usize>,
    pub hash: Option<usize>,
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
