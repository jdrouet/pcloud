use super::FileResponse;
use crate::common::RemoteFile;
use crate::request::{Error, Response};
use crate::PCloudApi;

impl PCloudApi {
    /// Rename a folder
    ///
    /// # Arguments
    ///
    /// * `file_id` - ID of the file to rename.
    /// * `name` - New name for the folder
    ///
    pub async fn rename_file(&self, file_id: usize, name: &str) -> Result<RemoteFile, Error> {
        let file_id = file_id.to_string();
        let params = vec![("fileid", file_id.as_str()), ("toname", name)];
        let result: Response<FileResponse> = self.get_request("renamefile", &params).await?;
        result.payload().map(|item| item.metadata)
    }

    /// Move a file
    ///
    /// # Arguments
    ///
    /// * `file_id` - ID of the file to move.
    /// * `to_folder_id` - ID of the folder to move the file in.
    ///
    pub async fn move_file(
        &self,
        folder_id: usize,
        to_folder_id: usize,
    ) -> Result<RemoteFile, Error> {
        let file_id = folder_id.to_string();
        let to_folder_id = to_folder_id.to_string();
        let params = vec![
            ("fileid", file_id.as_str()),
            ("tofolderid", to_folder_id.as_str()),
        ];
        let result: Response<FileResponse> = self.get_request("renamefile", &params).await?;
        result.payload().map(|item| item.metadata)
    }
}
