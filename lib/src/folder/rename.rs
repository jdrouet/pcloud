use super::FolderResponse;
use crate::entry::Folder;
use crate::error::Error;
use crate::http::PCloudHttpApi;
use crate::request::Response;

#[derive(Debug)]
pub enum Params {
    Rename {
        folder_id: usize,
        name: String,
    },
    Move {
        folder_id: usize,
        to_folder_id: usize,
    },
}

impl Params {
    pub fn new_rename<S: Into<String>>(folder_id: usize, name: S) -> Self {
        Self::Rename {
            folder_id,
            name: name.into(),
        }
    }

    pub fn new_move(folder_id: usize, to_folder_id: usize) -> Self {
        Self::Move {
            folder_id,
            to_folder_id,
        }
    }

    pub fn to_vec(&self) -> Vec<(&str, String)> {
        match self {
            Self::Rename { folder_id, name } => vec![
                ("folderid", folder_id.to_string()),
                ("toname", name.to_string()),
            ],
            Self::Move {
                folder_id,
                to_folder_id,
            } => vec![
                ("folderid", folder_id.to_string()),
                ("tofolderid", to_folder_id.to_string()),
            ],
        }
    }
}

impl PCloudHttpApi {
    /// Rename a folder
    ///
    /// # Arguments
    ///
    /// * `folder_id` - ID of the folder to rename.
    /// * `name` - New name for the folder
    ///
    pub async fn rename_folder(&self, params: &Params) -> Result<Folder, Error> {
        let result: Response<FolderResponse> =
            self.get_request("renamefolder", &params.to_vec()).await?;
        result.payload().map(|item| item.metadata)
    }
}
