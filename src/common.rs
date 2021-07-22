#[derive(Debug, serde::Deserialize, serde::Serialize)]
pub struct RemoteFile {
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
    #[serde(rename = "folderid")]
    pub folder_id: Option<usize>,
    #[serde(rename = "isshared")]
    pub is_shared: bool,
    #[serde(rename = "ismine")]
    pub is_mine: bool,
    pub name: String,
    pub contents: Option<Vec<RemoteFile>>,
}
