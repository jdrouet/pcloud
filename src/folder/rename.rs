use super::FolderResponse;
use crate::common::RemoteFile;
use crate::request::{Error, Response};
use crate::PCloudApi;

impl PCloudApi {
    pub async fn rename_folder(&self, folder_id: usize, name: &str) -> Result<RemoteFile, Error> {
        let folder_id = folder_id.to_string();
        let params = vec![("folderid", folder_id.as_str()), ("toname", name)];
        let result: Response<FolderResponse> = self.get_request("renamefolder", &params).await?;
        result.payload().map(|item| item.metadata)
    }

    pub async fn move_folder(
        &self,
        folder_id: usize,
        to_folder_id: usize,
    ) -> Result<RemoteFile, Error> {
        let folder_id = folder_id.to_string();
        let to_folder_id = to_folder_id.to_string();
        let params = vec![
            ("folderid", folder_id.as_str()),
            ("tofolderid", to_folder_id.as_str()),
        ];
        let result: Response<FolderResponse> = self.get_request("renamefolder", &params).await?;
        result.payload().map(|item| item.metadata)
    }
}
