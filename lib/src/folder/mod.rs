use std::{borrow::Cow, cmp::Ordering};

use serde::ser::SerializeStruct;

pub mod create;
pub mod delete;
pub mod list;
pub mod movefolder;
pub mod rename;

pub const ROOT: u64 = 0;

/// Internal response structure for folder-related API calls.
#[derive(Debug, serde::Deserialize)]
pub(crate) struct FolderResponse {
    /// Metadata of the folder being returned.
    pub metadata: Folder,
}

/// Enumeration for identifying a folder by either its path or folder ID.
#[derive(Debug)]
pub enum FolderIdentifier<'a> {
    /// A folder is identified by its path.
    Path(Cow<'a, str>),

    /// A folder is identified by its unique folder ID.
    FolderId(u64),
}

impl<'a> FolderIdentifier<'a> {
    /// Create a folder identifier using a path.
    #[inline]
    pub fn path<P: Into<Cow<'a, str>>>(value: P) -> Self {
        Self::Path(value.into())
    }

    /// Create a folder identifier using a folder ID.
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

/// A structure representing a folder in pCloud.
#[derive(Clone, Debug, serde::Deserialize, serde::Serialize)]
pub struct Folder {
    /// Metadata common to all entries in pCloud.
    #[serde(flatten)]
    pub base: crate::entry::EntryBase,

    /// The unique folder ID.
    #[serde(rename = "folderid")]
    pub folder_id: u64,

    /// A list of contents inside the folder (files and subfolders).
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
    /// Finds an entry (file or folder) by its name inside the folder.
    ///
    /// # Arguments
    ///
    /// * `name` - The name of the entry to search for.
    ///
    /// # Returns
    ///
    /// An optional reference to the entry if it exists in the folder.
    ///
    /// # Examples
    ///
    /// ```rust
    /// # async fn example(folder: &pcloud::folder::Folder) {
    /// if let Some(entry) = folder.find_entry("example.txt") {
    ///     println!("Found entry: {:?}", entry.base().name);
    /// }
    /// # }
    /// ```
    pub fn find_entry(&self, name: &str) -> Option<&crate::entry::Entry> {
        self.contents
            .as_ref()
            .and_then(|list| list.iter().find(|item| item.base().name == name))
    }

    /// Finds a file by its name inside the folder.
    ///
    /// # Arguments
    ///
    /// * `name` - The name of the file to search for.
    ///
    /// # Returns
    ///
    /// An optional reference to the file if it exists in the folder.
    pub fn find_file(&self, name: &str) -> Option<&crate::file::File> {
        self.contents.as_ref().and_then(|list| {
            list.iter()
                .filter_map(|item| item.as_file())
                .find(|item| item.base.name == name)
        })
    }

    /// Finds a subfolder by its name inside the folder.
    ///
    /// # Arguments
    ///
    /// * `name` - The name of the subfolder to search for.
    ///
    /// # Returns
    ///
    /// An optional reference to the subfolder if it exists in the folder.
    pub fn find_folder(&self, name: &str) -> Option<&Folder> {
        self.contents.as_ref().and_then(|list| {
            list.iter()
                .filter_map(|item| item.as_folder())
                .find(|item| item.base.name == name)
        })
    }
}

/// Internal wrapper for a folder identifier used in API requests.
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
