use super::FileResponse;
use crate::binary::{BinaryClient, Value as BinaryValue};
use crate::entry::File;
use crate::error::Error;
use crate::http::HttpClient;
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

    fn to_http_params(&self) -> Vec<(&str, String)> {
        vec![
            ("fileid", self.file_id.to_string()),
            ("tofolderid", self.to_folder_id.to_string()),
        ]
    }

    fn to_binary_params(&self) -> Vec<(&str, BinaryValue)> {
        vec![
            ("fileid", BinaryValue::Number(self.file_id as u64)),
            ("tofolderid", BinaryValue::Number(self.to_folder_id as u64)),
        ]
    }
}

impl HttpClient {
    pub async fn copy_file(&self, params: &Params) -> Result<File, Error> {
        let result: Response<FileResponse> = self
            .get_request("copyfile", &params.to_http_params())
            .await?;
        result.payload().map(|item| item.metadata)
    }
}

impl BinaryClient {
    pub fn copy_file(&mut self, params: &Params) -> Result<File, Error> {
        let result = self.send_command("copyfile", &params.to_binary_params())?;
        let result: Response<FileResponse> = serde_json::from_value(result)?;
        result.payload().map(|item| item.metadata)
    }
}
