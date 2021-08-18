use super::FileResponse;
use crate::entry::File;
use crate::error::Error;
use crate::http::PCloudHttpApi;
use crate::request::Response;

pub struct Params {
    file_id: usize,
    to_folder_id: usize,
}

impl Params {
    pub fn new(file_id: usize, to_folder_id: usize) -> Self {
        Self {
            file_id,
            to_folder_id,
        }
    }

    pub fn to_vec(&self) -> Vec<(&str, String)> {
        vec![
            ("fileid", self.file_id.to_string()),
            ("tofolderid", self.to_folder_id.to_string()),
        ]
    }
}

impl PCloudHttpApi {
    pub async fn copy_file(&self, params: &Params) -> Result<File, Error> {
        let result: Response<FileResponse> = self.get_request("copyfile", &params.to_vec()).await?;
        result.payload().map(|item| item.metadata)
    }
}
