use chrono::{DateTime, Utc};

/// A set of shared fields between [`File`](File) and [`Folder`](Folder).
#[derive(Clone, Debug, serde::Deserialize, serde::Serialize)]
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
