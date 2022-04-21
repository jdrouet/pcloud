use super::FileResponse;
use crate::binary::{BinaryClient, Value as BinaryValue};
use crate::entry::File;
use crate::error::Error;
use crate::http::HttpClient;
use crate::prelude::HttpCommand;
use crate::request::Response;

#[derive(Debug)]
pub struct FileCopyCommand {
    file_id: u64,
    to_folder_id: u64,
}

impl FileCopyCommand {
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

#[async_trait::async_trait(?Send)]
impl HttpCommand for FileCopyCommand {
    type Output = File;

    async fn execute(self, client: &HttpClient) -> Result<Self::Output, Error> {
        let result: Response<FileResponse> = client
            .get_request("copyfile", &self.to_http_params())
            .await?;
        result.payload().map(|item| item.metadata)
    }
}

impl BinaryClient {
    #[tracing::instrument(skip(self))]
    pub fn copy_file(&mut self, params: &FileCopyCommand) -> Result<File, Error> {
        let result = self.send_command("copyfile", &params.to_binary_params())?;
        let result: Response<FileResponse> = serde_json::from_value(result)?;
        result.payload().map(|item| item.metadata)
    }
}
