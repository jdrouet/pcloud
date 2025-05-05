use std::cmp::Ordering;

use chrono::{DateTime, Utc};

/// A set of shared fields between [`File`](crate::file::File) and [`Folder`](crate::folder::Folder).
#[derive(Clone, Debug, PartialEq, Eq, serde::Deserialize, serde::Serialize)]
pub struct EntryBase {
    #[serde(with = "crate::date")]
    pub created: DateTime<Utc>,
    #[serde(with = "crate::date")]
    pub modified: DateTime<Utc>,
    #[serde(rename = "parentfolderid")]
    pub parent_folder_id: Option<u64>,
    pub icon: String,
    pub id: String,
    pub name: String,
    pub path: Option<String>,
    pub thumb: bool,
    #[serde(rename = "isshared")]
    pub is_shared: bool,
    #[serde(rename = "ismine")]
    pub is_mine: bool,
}

/// The representation of what can be returned by the PCloud API, a file or a folder.
#[derive(Clone, Debug, Eq, PartialEq, serde::Deserialize, serde::Serialize)]
#[serde(untagged)]
#[allow(clippy::large_enum_variant)]
pub enum Entry {
    File(crate::file::File),
    Folder(crate::folder::Folder),
}
impl PartialOrd for Entry {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for Entry {
    fn cmp(&self, other: &Self) -> Ordering {
        match (self, other) {
            (Self::File(self_file), Self::File(other_file)) => self_file.cmp(other_file),
            (Self::File(_), Self::Folder(_)) => Ordering::Greater,
            (Self::Folder(self_folder), Self::Folder(other_folder)) => {
                self_folder.cmp(other_folder)
            }
            (Self::Folder(_), Self::File(_)) => Ordering::Less,
        }
    }
}

impl From<crate::file::File> for Entry {
    fn from(value: crate::file::File) -> Self {
        Self::File(value)
    }
}

impl From<crate::folder::Folder> for Entry {
    fn from(value: crate::folder::Folder) -> Self {
        Self::Folder(value)
    }
}

impl Entry {
    pub fn base(&self) -> &EntryBase {
        match self {
            Self::File(file) => &file.base,
            Self::Folder(folder) => &folder.base,
        }
    }

    pub fn file_id(&self) -> Option<u64> {
        match self {
            Self::File(item) => Some(item.file_id),
            _ => None,
        }
    }

    pub fn is_file(&self) -> bool {
        matches!(self, Self::File(_))
    }

    pub fn into_file(self) -> Option<crate::file::File> {
        match self {
            Self::File(value) => Some(value),
            _ => None,
        }
    }

    pub fn as_file(&self) -> Option<&crate::file::File> {
        match self {
            Self::File(value) => Some(value),
            _ => None,
        }
    }

    pub fn folder_id(&self) -> Option<u64> {
        match self {
            Self::Folder(item) => Some(item.folder_id),
            _ => None,
        }
    }

    pub fn is_folder(&self) -> bool {
        matches!(self, Self::Folder(_))
    }

    pub fn into_folder(self) -> Option<crate::folder::Folder> {
        match self {
            Self::Folder(value) => Some(value),
            _ => None,
        }
    }

    pub fn as_folder(&self) -> Option<&crate::folder::Folder> {
        match self {
            Self::Folder(value) => Some(value),
            _ => None,
        }
    }
}
