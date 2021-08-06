use std::cmp::Ordering;

#[derive(Debug, Eq, serde::Deserialize, serde::Serialize)]
pub struct File {
    // TODO replace by chrono
    pub created: String,
    #[serde(rename = "parentfolderid")]
    pub parent_folder_id: Option<usize>,
    pub icon: String,
    pub id: String,
    pub path: Option<String>,
    // TODO replace by chrono
    pub modified: String,
    pub thumb: bool,
    #[serde(rename = "fileid")]
    pub file_id: usize,
    #[serde(rename = "isshared")]
    pub is_shared: bool,
    #[serde(rename = "ismine")]
    pub is_mine: bool,
    pub name: String,
    pub size: Option<usize>,
    pub hash: Option<usize>,
    #[serde(rename = "contenttype")]
    pub content_type: Option<String>,
}

#[derive(Debug, Eq, serde::Deserialize, serde::Serialize)]
pub struct Folder {
    // TODO replace by chrono
    pub created: String,
    #[serde(rename = "parentfolderid")]
    pub parent_folder_id: Option<usize>,
    pub icon: String,
    pub id: String,
    pub path: Option<String>,
    // TODO replace by chrono
    pub modified: String,
    pub thumb: bool,
    #[serde(rename = "folderid")]
    pub folder_id: usize,
    #[serde(rename = "isshared")]
    pub is_shared: bool,
    #[serde(rename = "ismine")]
    pub is_mine: bool,
    pub name: String,
    pub size: Option<usize>,
    pub hash: Option<usize>,
    pub contents: Option<Vec<Entry>>,
}

macro_rules! entry_field {
    ($field:ident, $output:ty) => {
        entry_field!($field, $output, $field);
    };
    ($fname:ident, $output:ty, $field:ident) => {
        impl Entry {
            pub fn $fname(&self) -> $output {
                match self {
                    Self::File(item) => item.$field,
                    Self::Folder(item) => item.$field,
                }
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
                match self {
                    Self::File(item) => &item.$field,
                    Self::Folder(item) => &item.$field,
                }
            }
        }
    };
}

#[derive(Debug, Eq, PartialEq, serde::Deserialize, serde::Serialize)]
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
entry_field_ref!(created, &str);
entry_field_ref!(modified, &str);
entry_field_ref!(icon, &str);
entry_field_ref!(parent_folder_id, &Option<usize>);
entry_field!(is_shared, bool);
entry_field!(is_mine, bool);

impl Entry {
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

impl PartialEq for File {
    fn eq(&self, other: &Self) -> bool {
        self.file_id == other.file_id
    }
}

impl PartialOrd for File {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for File {
    fn cmp(&self, other: &Self) -> Ordering {
        self.name.cmp(&other.name)
    }
}

impl PartialEq for Folder {
    fn eq(&self, other: &Self) -> bool {
        self.folder_id == other.folder_id
    }
}

impl PartialOrd for Folder {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for Folder {
    fn cmp(&self, other: &Self) -> Ordering {
        self.name.cmp(&other.name)
    }
}

impl PartialOrd for Entry {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for Entry {
    fn cmp(&self, other: &Self) -> Ordering {
        match self {
            Self::Folder(self_folder) => match other {
                Self::Folder(other_folder) => self_folder.cmp(other_folder),
                Self::File(_) => Ordering::Less,
            },
            Self::File(self_file) => match other {
                Self::Folder(_) => Ordering::Greater,
                Self::File(other_file) => self_file.cmp(other_file),
            },
        }
    }
}
