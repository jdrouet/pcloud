use std::{borrow::Cow, cmp::Ordering};

use serde::ser::SerializeStruct;

pub mod create;
pub mod delete;
pub mod list;
pub mod movefolder;
pub mod rename;

pub const ROOT: u64 = 0;

#[derive(Debug, serde::Deserialize)]
pub(crate) struct FolderResponse {
    pub metadata: Folder,
}

#[derive(Debug)]
pub enum FolderIdentifier<'a> {
    Path(Cow<'a, str>),
    FolderId(u64),
}

impl<'a> FolderIdentifier<'a> {
    #[inline]
    pub fn path<P: Into<Cow<'a, str>>>(value: P) -> Self {
        Self::Path(value.into())
    }

    #[inline]
    pub fn folder_id(value: u64) -> Self {
        Self::FolderId(value)
    }
}

impl Default for FolderIdentifier<'_> {
    fn default() -> Self {
        Self::FolderId(0)
    }
}

impl<'a> From<&'a str> for FolderIdentifier<'a> {
    fn from(value: &'a str) -> Self {
        Self::Path(Cow::Borrowed(value))
    }
}

impl<'a> From<Cow<'a, str>> for FolderIdentifier<'a> {
    fn from(value: Cow<'a, str>) -> Self {
        Self::Path(value)
    }
}

impl<'a> From<&'a String> for FolderIdentifier<'a> {
    fn from(value: &'a String) -> Self {
        Self::Path(Cow::Borrowed(value.as_str()))
    }
}

impl From<String> for FolderIdentifier<'_> {
    fn from(value: String) -> Self {
        Self::Path(Cow::Owned(value))
    }
}

impl From<u64> for FolderIdentifier<'_> {
    fn from(value: u64) -> Self {
        Self::FolderId(value)
    }
}

impl serde::Serialize for FolderIdentifier<'_> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let mut builder = serializer.serialize_struct(stringify!(FolderIdentifier), 1)?;
        match self {
            Self::FolderId(folder_id) => {
                builder.serialize_field("folderid", folder_id)?;
            }
            Self::Path(path) => {
                builder.serialize_field("path", path)?;
            }
        }
        builder.end()
    }
}

/// A structure reprensenting a folder on PCloud
#[derive(Clone, Debug, serde::Deserialize, serde::Serialize)]
pub struct Folder {
    #[serde(flatten)]
    pub base: crate::entry::EntryBase,
    #[serde(rename = "folderid")]
    pub folder_id: u64,
    pub contents: Option<Vec<crate::entry::Entry>>,
}

impl Eq for Folder {}

impl PartialEq for Folder {
    fn eq(&self, other: &Self) -> bool {
        self.base.id.eq(&other.base.id)
    }
}

impl Ord for Folder {
    fn cmp(&self, other: &Self) -> Ordering {
        self.base.name.cmp(&other.base.name)
    }
}

impl PartialOrd for Folder {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Folder {
    pub fn find_entry(&self, name: &str) -> Option<&crate::entry::Entry> {
        self.contents
            .as_ref()
            .and_then(|list| list.iter().find(|item| item.base().name == name))
    }

    pub fn find_file(&self, name: &str) -> Option<&crate::file::File> {
        self.contents.as_ref().and_then(|list| {
            list.iter()
                .filter_map(|item| item.as_file())
                .find(|item| item.base.name == name)
        })
    }

    pub fn find_folder(&self, name: &str) -> Option<&Folder> {
        self.contents.as_ref().and_then(|list| {
            list.iter()
                .filter_map(|item| item.as_folder())
                .find(|item| item.base.name == name)
        })
    }
}

pub(crate) struct ToFolderIdentifier<'a>(pub FolderIdentifier<'a>);

impl serde::Serialize for ToFolderIdentifier<'_> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let mut builder = serializer.serialize_struct(stringify!(FolderIdentifier), 1)?;
        match self.0 {
            FolderIdentifier::FolderId(ref folder_id) => {
                builder.serialize_field("tofolderid", folder_id)?;
            }
            FolderIdentifier::Path(ref path) => {
                builder.serialize_field("topath", path)?;
            }
        }
        builder.end()
    }
}
