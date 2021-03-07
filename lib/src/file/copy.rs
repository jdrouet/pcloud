use super::FileResponse;
use crate::binary::{BinaryClient, Value as BinaryValue};
use crate::entry::File;
use crate::error::Error;
use crate::http::HttpClient;
use crate::request::Response;

#[derive(Debug)]
pub struct Params {
    file_id: u64,
    to_folder_id: u64,
}

impl Params {
    pub fn new(file_id: u64, to_folder_id: u64) -> Self {
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
            ("fileid", BinaryValue::Number(self.file_id)),
            ("tofolderid", BinaryValue::Number(self.to_folder_id)),
        ]
    }
}

impl HttpClient {
    #[tracing::instrument(skip(self))]
    pub async fn copy_file(&self, params: &Params) -> Result<File, Error> {
        let result: Response<FileResponse> = self
            .get_request("copyfile", &params.to_http_params())
            .await?;
        result.payload().map(|item| item.metadata)
    }
}

impl BinaryClient {
    #[tracing::instrument(skip(self))]
    pub fn copy_file(&mut self, params: &Params) -> Result<File, Error> {
        let result = self.send_command("copyfile", &params.to_binary_params())?;
        let result: Response<FileResponse> = serde_json::from_value(result)?;
        result.payload().map(|item| item.metadata)
    }
}
