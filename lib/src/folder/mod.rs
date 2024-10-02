use std::borrow::Cow;

pub mod create;
pub mod delete;
pub mod list;
pub mod rename;

pub const ROOT: u64 = 0;

#[cfg(feature = "client-http")]
#[derive(Debug, serde::Deserialize)]
pub(crate) struct FolderResponse {
    pub metadata: crate::entry::Folder,
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

impl<'a> Default for FolderIdentifier<'a> {
    fn default() -> Self {
        Self::FolderId(0)
    }
}

impl<'a> From<&'a str> for FolderIdentifier<'a> {
    fn from(value: &'a str) -> Self {
        Self::Path(Cow::Borrowed(value))
    }
}

impl<'a> From<String> for FolderIdentifier<'a> {
    fn from(value: String) -> Self {
        Self::Path(Cow::Owned(value))
    }
}

impl<'a> From<u64> for FolderIdentifier<'a> {
    fn from(value: u64) -> Self {
        Self::FolderId(value)
    }
}

#[cfg(feature = "client-http")]
#[derive(serde::Serialize)]
#[serde(untagged)]
pub(crate) enum FolderIdentifierParam<'a> {
    Path { path: Cow<'a, str> },
    FolderId { folderid: u64 },
}

#[cfg(feature = "client-http")]
impl<'a> From<FolderIdentifier<'a>> for FolderIdentifierParam<'a> {
    fn from(value: FolderIdentifier<'a>) -> Self {
        match value {
            FolderIdentifier::FolderId(folderid) => Self::FolderId { folderid },
            FolderIdentifier::Path(path) => Self::Path { path },
        }
    }
}
