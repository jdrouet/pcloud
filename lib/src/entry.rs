use chrono::prelude::{DateTime, Utc};
use std::cmp::Ordering;

#[derive(Clone, Debug, serde::Deserialize, serde::Serialize)]
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

#[derive(Clone, Debug, serde::Deserialize, serde::Serialize)]
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

#[derive(Clone, Debug, serde::Deserialize, serde::Serialize)]
pub struct Folder {
    #[serde(flatten)]
    pub base: EntryBase,
    #[serde(rename = "folderid")]
    pub folder_id: usize,
    pub contents: Option<Vec<Entry>>,
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
    pub fn find_entry(&self, name: &str) -> Option<&Entry> {
        self.contents
            .as_ref()
            .and_then(|list| list.iter().find(|item| item.base().name == name))
    }

    pub fn find_file(&self, name: &str) -> Option<&File> {
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

#[derive(Clone, Debug, Eq, PartialEq, serde::Deserialize, serde::Serialize)]
#[serde(untagged)]
pub enum Entry {
    File(File),
    Folder(Folder),
}

impl PartialOrd for Entry {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for Entry {
    fn cmp(&self, other: &Self) -> Ordering {
        match self {
            Self::File(self_file) => match other {
                Self::File(other_file) => self_file.cmp(other_file),
                Self::Folder(_) => Ordering::Greater,
            },
            Self::Folder(self_folder) => match other {
                Self::File(_) => Ordering::Less,
                Self::Folder(other_folder) => self_folder.cmp(other_folder),
            },
        }
    }
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

    pub fn into_file(self) -> Option<File> {
        match self {
            Self::File(value) => Some(value),
            _ => None,
        }
    }

    pub fn as_file(&self) -> Option<&File> {
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

    pub fn into_folder(self) -> Option<Folder> {
        match self {
            Self::Folder(value) => Some(value),
            _ => None,
        }
    }

    pub fn as_folder(&self) -> Option<&Folder> {
        match self {
            Self::Folder(value) => Some(value),
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_file(id: usize, name: &str) -> File {
        File {
            base: EntryBase {
                created: Utc::now(),
                modified: Utc::now(),
                parent_folder_id: None,
                icon: "".into(),
                id: format!("f{}", id),
                name: name.into(),
                path: None,
                thumb: false,
                is_shared: false,
                is_mine: false,
            },
            file_id: id,
            size: Some(42),
            hash: Some(42),
            content_type: None,
        }
    }

    fn create_folder(id: usize, name: &str) -> Folder {
        Folder {
            base: EntryBase {
                created: Utc::now(),
                modified: Utc::now(),
                parent_folder_id: None,
                icon: "".into(),
                id: format!("d{}", id),
                name: name.into(),
                path: None,
                thumb: false,
                is_shared: false,
                is_mine: false,
            },
            folder_id: id,
            contents: None,
        }
    }

    #[test]
    fn sorting() {
        let mut data: Vec<Entry> = vec![
            create_file(1, "cccc").into(),
            create_folder(2, "dddd").into(),
            create_file(3, "aaaa").into(),
            create_folder(4, "eeee").into(),
        ];
        data.sort();
        let ids: Vec<_> = data.iter().map(|item| item.base().id.clone()).collect();
        assert_eq!(ids, vec!["d2", "d4", "f3", "f1"]);
    }
}
