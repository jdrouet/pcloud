use super::FileResponse;
use crate::entry::File;
use crate::error::Error;
use crate::http::PCloudApi;
use crate::request::Response;

impl PCloudApi {
    /// Copy a file
    ///
    /// # Arguments
    ///
    /// * `file_id` - ID of the file to rename.
    /// * `to_folder_id` - ID of the folder to copy to.
    ///
    pub async fn copy_file(&self, file_id: usize, to_folder_id: usize) -> Result<File, Error> {
        let file_id = file_id.to_string();
        let to_folder_id = to_folder_id.to_string();
        let params = vec![
            ("fileid", file_id.as_str()),
            ("tofolderid", to_folder_id.as_str()),
        ];
        let result: Response<FileResponse> = self.get_request("copyfile", &params).await?;
        result.payload().map(|item| item.metadata)
    }
}
