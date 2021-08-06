use chrono::prelude::{DateTime, Utc};

#[derive(Debug, serde::Deserialize, serde::Serialize)]
pub struct EntryBase {
    #[serde(with = "crate::date")]
    pub created: DateTime<Utc>,
    #[serde(with = "crate::date")]
    pub modified: DateTime<Utc>,
    #[serde(rename = "parentfolderid")]
    pub parent_folder_id: Option<usize>,
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

#[derive(Debug, serde::Deserialize, serde::Serialize)]
pub struct File {
    #[serde(flatten)]
    pub base: EntryBase,
    #[serde(rename = "fileid")]
    pub file_id: usize,
    pub size: Option<usize>,
    pub hash: Option<usize>,
    #[serde(rename = "contenttype")]
    pub content_type: Option<String>,
}

#[derive(Debug, serde::Deserialize, serde::Serialize)]
pub struct Folder {
    #[serde(flatten)]
    pub base: EntryBase,
    #[serde(rename = "folderid")]
    pub folder_id: usize,
    pub contents: Option<Vec<Entry>>,
}

macro_rules! entry_field {
    ($field:ident, $output:ty) => {
        entry_field!($field, $output, $field);
    };
    ($fname:ident, $output:ty, $field:ident) => {
        impl Entry {
            pub fn $fname(&self) -> $output {
                self.base().$field
            }
        }
    };
}

macro_rules! entry_field_ref {
    ($field:ident, $output:ty) => {
        entry_field_ref!($field, $output, $field);
    };
    ($fname:ident, $output:ty, $field:ident) => {
        impl Entry {
            pub fn $fname(&self) -> $output {
                &self.base().$field
            }
        }
    };
}

#[derive(Debug, serde::Deserialize, serde::Serialize)]
#[serde(untagged)]
pub enum Entry {
    File(File),
    Folder(Folder),
}

impl From<File> for Entry {
    fn from(value: File) -> Self {
        Self::File(value)
    }
}

impl From<Folder> for Entry {
    fn from(value: Folder) -> Self {
        Self::Folder(value)
    }
}

entry_field_ref!(id, &str);
entry_field_ref!(name, &str);
entry_field_ref!(created, &DateTime<Utc>);
entry_field_ref!(modified, &DateTime<Utc>);
entry_field_ref!(icon, &str);
entry_field_ref!(parent_folder_id, &Option<usize>);
entry_field!(is_shared, bool);
entry_field!(is_mine, bool);

impl Entry {
    pub fn base(&self) -> &EntryBase {
        match self {
            Self::File(file) => &file.base,
            Self::Folder(folder) => &folder.base,
        }
    }

    pub fn file_id(&self) -> Option<usize> {
        match self {
            Self::File(item) => Some(item.file_id),
            _ => None,
        }
    }

    pub fn is_file(&self) -> bool {
        matches!(self, Self::File(_))
    }

    pub fn as_file(self) -> Option<File> {
        match self {
            Self::File(value) => Some(value),
            _ => None,
        }
    }

    pub fn folder_id(&self) -> Option<usize> {
        match self {
            Self::Folder(item) => Some(item.folder_id),
            _ => None,
        }
    }

    pub fn is_folder(&self) -> bool {
        matches!(self, Self::Folder(_))
    }

    pub fn as_folder(self) -> Option<Folder> {
        match self {
            Self::Folder(value) => Some(value),
            _ => None,
        }
    }
}

/*
#[derive(Debug, serde::Deserialize, serde::Serialize)]
struct RemoteEntry {
    // TODO replace by chrono
    pub created: String,
    #[serde(rename = "isfolder")]
    pub is_folder: bool,
    #[serde(rename = "parentfolderid")]
    pub parent_folder_id: Option<usize>,
    pub icon: String,
    pub id: String,
    pub path: Option<String>,
    // TODO replace by chrono
    pub modified: String,
    pub thumb: bool,
    #[serde(rename = "fileid")]
    pub file_id: Option<usize>,
    #[serde(rename = "folderid")]
    pub folder_id: Option<usize>,
    #[serde(rename = "isshared")]
    pub is_shared: bool,
    #[serde(rename = "ismine")]
    pub is_mine: bool,
    pub name: String,
    pub size: Option<usize>,
    pub hash: Option<usize>,
    #[serde(rename = "contenttype")]
    pub content_type: Option<String>,
    pub contents: Option<Vec<RemoteEntry>>,
}
*/
