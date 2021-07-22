#[derive(Debug, serde::Deserialize, serde::Serialize)]
pub struct RemoteFile {
    // TODO replace by chrono
    created: String,
    #[serde(rename = "isfolder")]
    is_folder: bool,
    #[serde(rename = "parentfolderid")]
    parent_folder_id: Option<usize>,
    icon: String,
    id: String,
    path: Option<String>,
    // TODO replace by chrono
    modified: String,
    thumb: bool,
    #[serde(rename = "folderid")]
    folder_id: Option<usize>,
    #[serde(rename = "isshared")]
    is_shared: bool,
    #[serde(rename = "ismine")]
    is_mine: bool,
    name: String,
    contents: Option<Vec<RemoteFile>>,
}
